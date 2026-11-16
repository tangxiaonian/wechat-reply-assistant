import { computed, ref } from "vue";
import { defineStore } from "pinia";
import * as bridge from "../bridge";
import { activeProfileItems } from "../domain/profiles";
import { demoReplies } from "../domain/replies";
import defaultTones from "../../../shared/tone-defaults.json";
import type {
  ProfileAction,
  ProfileTab,
  ProviderSummary,
  Task,
  Tone,
  Workspace,
  ToneTemplate,
  SyncSettings,
} from "../types/workspace";

export const useWorkspace = defineStore("workspace", () => {
  const workspace = ref<Workspace | null>(null);
  const selectedId = ref("demo-friend");
  const selectedMessageIds = ref<string[]>([]);
  const replyTargetId = ref<string | null>(null);
  const toneTemplates = ref<ToneTemplate[]>(
    structuredClone(defaultTones) as ToneTemplate[],
  );
  const selectedToneId = ref<string | null>(null);
  let privateProfileOpen = true;
  const profileOpen = ref(true);
  const profileTab = ref<ProfileTab>("info");
  const settingsOpen = ref(false);
  const historyOpen = ref(false);
  const backgroundEnabled = ref(false);
  const refreshingContext = ref(false);
  const contextReadAt = ref(Date.now());
  const syncPreferences = ref<SyncSettings>({
    enabled: false,
    intervalSeconds: 30,
  });
  const settingsTab = ref<"connection" | "tones" | "sync">("connection");
  const contextOpen = ref(false);
  const task = ref<Task>("reply");
  const tone = ref<Tone>("自然");
  const customTone = ref("");
  const background = ref("");
  const reference = ref(true);
  const useModel = ref(false);
  const manualTranscript = ref<string | null>(null);
  const candidates = ref<string[]>([]);
  const selectedCandidate = ref(-1);
  const draft = ref("");
  const generated = ref(false);
  const generatedMode = ref<"demo" | "model">("demo");
  const generating = ref(false);
  const mutationBusy = ref(false);
  const loading = ref(true);
  const pinned = ref(false);
  const toast = ref("");
  const error = ref("");
  const provider = ref<ProviderSummary>({
    name: "",
    baseUrl: "",
    model: "",
    configured: false,
    consent: false,
  });
  let requestId: string | null = null;
  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  const contact = computed(() =>
    workspace.value?.contacts.find((item) => item.id === selectedId.value),
  );
  const profile = computed(() => workspace.value?.profiles[selectedId.value]);
  const isGroup = computed(() => contact.value?.kind === "group");
  const availableTones = computed(() =>
    toneTemplates.value.filter((entry) => entry.enabled),
  );
  const chosenTone = computed(
    () =>
      availableTones.value.find((entry) => entry.id === selectedToneId.value) ??
      availableTones.value.find((entry) => entry.baseTone === tone.value) ??
      availableTones.value[0],
  );
  const activeItems = computed(() =>
    profile.value ? activeProfileItems(profile.value) : [],
  );
  const conflictCount = computed(
    () =>
      profile.value?.proposals.filter((item) => item.kind === "conflict")
        .length ?? 0,
  );
  const observationCount = computed(
    () =>
      profile.value?.proposals.filter((item) => item.kind === "observation")
        .length ?? 0,
  );

  /** Displays one brief, accessible confirmation; errors remain visible until corrected. */
  function notify(message: string): void {
    toast.value = message;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toast.value = "";
    }, 4200);
  }

  async function initialize(): Promise<void> {
    try {
      [workspace.value, provider.value] = await Promise.all([
        bridge.loadWorkspace(),
        bridge.getProvider(),
      ]);
    } catch (failure) {
      error.value = bridge.errorMessage(failure);
    } finally {
      loading.value = false;
    }
  }

  /** Cancels in-flight work before clearing ephemeral information on a contact change. */
  async function cancel(): Promise<void> {
    const previous = requestId;
    requestId = null;
    generating.value = false;
    if (previous) {
      await bridge.cancelReply(previous).catch(() => {});
    }
  }

  async function selectContact(id: string): Promise<void> {
    if (id === selectedId.value) {
      return;
    }
    await cancel();
    const wasGroup = isGroup.value;
    if (!wasGroup) {
      privateProfileOpen = profileOpen.value;
    }
    selectedId.value = id;
    selectedMessageIds.value = [];
    replyTargetId.value = null;
    reference.value = !isGroup.value;
    profileOpen.value = isGroup.value ? false : privateProfileOpen;
    contextOpen.value = isGroup.value;
    if (wasGroup !== isGroup.value) {
      await bridge
        .windowAction(profileOpen.value ? "expand" : "collapse")
        .catch(() => {});
    }
    background.value = "";
    backgroundEnabled.value = false;
    customTone.value = "";
    manualTranscript.value = null;
    candidates.value = [];
    draft.value = "";
    selectedCandidate.value = -1;
    generated.value = false;
    error.value = "";
    profileTab.value = "info";
    task.value = "reply";
  }

  async function toggleProfile(): Promise<void> {
    if (isGroup.value) {
      return;
    }
    profileOpen.value = !profileOpen.value;
    await bridge
      .windowAction(profileOpen.value ? "expand" : "collapse")
      .catch((failure: unknown) => {
        notify(bridge.errorMessage(failure));
      });
  }

  async function mutate(action: ProfileAction): Promise<void> {
    if (
      workspace.value?.contacts.find((entry) => entry.id === action.contactId)
        ?.kind === "group"
    ) {
      throw new Error("群聊不创建或使用画像");
    }
    mutationBusy.value = true;
    try {
      workspace.value = await bridge.profileAction(action);
      notify(
        action.type === "delete"
          ? action.excludeFuture
            ? "已删除，并排除该事项的后续记录"
            : "已删除资料及关联内容记录"
          : action.type === "demo_update"
            ? "已模拟加入一条偏好，原有资料保持不变"
            : "已保存到这个联系人的本地档案",
      );
    } catch (failure) {
      error.value = bridge.errorMessage(failure);
      throw failure;
    } finally {
      mutationBusy.value = false;
    }
  }

  /** Freezes the selected conversation and ignores results from a cancelled or replaced request. */
  async function generate(): Promise<void> {
    if (!contact.value || generating.value) {
      return;
    }
    error.value = "";
    if (isGroup.value && !selectedMessageIds.value.length) {
      error.value = "先选中至少一条要回复的群消息";
      historyOpen.value = true;
      return;
    }
    const template = chosenTone.value;
    if (!template) {
      error.value = "请先在设置中开启至少一种语气";
      return;
    }
    const id = crypto.randomUUID();
    const modelMode = useModel.value;
    requestId = id;
    generating.value = true;
    const input = {
      requestId: id,
      contactId: selectedId.value,
      task: task.value,
      tone: template?.name ?? tone.value,
      customTone: [template?.instruction, customTone.value]
        .filter(Boolean)
        .join("\n本次补充："),
      background: backgroundEnabled.value ? background.value : "",
      useProfile: !isGroup.value && reference.value,
      manualTranscript: manualTranscript.value,
      selectedMessageIds: [...selectedMessageIds.value],
      replyTargetId: replyTargetId.value,
    };
    try {
      if (
        modelMode &&
        (!provider.value.configured || !provider.value.consent)
      ) {
        settingsOpen.value = true;
        throw new Error("请先连接模型，并确认本次回复的处理范围");
      }
      const result = modelMode
        ? await bridge.generateReply(input)
        : {
            candidates: demoReplies(contact.value, {
              ...input,
              tone: template.baseTone ?? tone.value,
            }),
            mode: "demo" as const,
          };
      if (requestId !== id) {
        return;
      }
      candidates.value = result.candidates;
      generatedMode.value = result.mode;
      generated.value = true;
      selectedCandidate.value = -1;
      draft.value = "";
      if (!modelMode && (customTone.value || !template.baseTone)) {
        notify("演示展示预设候选；自定义语气在连接模型后生效");
      }
    } catch (failure) {
      if (requestId === id) {
        error.value = bridge.errorMessage(failure);
      }
    } finally {
      if (requestId === id) {
        generating.value = false;
        requestId = null;
      }
    }
  }

  /** Reloads the current demo source; it must not claim to have fetched live WeChat messages. */
  async function refreshContext(automatic = false): Promise<void> {
    if (refreshingContext.value || (automatic && generating.value)) {
      return;
    }
    if (manualTranscript.value) {
      if (!automatic) {
        notify("手动片段请重新粘贴；示例刷新不会覆盖你的内容");
      }
      return;
    }
    refreshingContext.value = true;
    try {
      if (!automatic) {
        await cancel();
      }
      workspace.value = await bridge.loadWorkspace();
      const ids = new Set(contact.value?.messages.map((message) => message.id));
      selectedMessageIds.value = selectedMessageIds.value.filter((id) =>
        ids.has(id),
      );
      contextReadAt.value = Date.now();
      if (!automatic) {
        notify("示例已重新载入，当前尚未连接真实微信消息；现有草稿已保留");
      }
    } catch (failure) {
      error.value = bridge.errorMessage(failure);
    } finally {
      refreshingContext.value = false;
    }
  }

  function chooseCandidate(index: number): void {
    selectedCandidate.value = index;
    draft.value = candidates.value[index] ?? "";
  }
  /** Changing the selected group messages or addressee invalidates the previous draft. */
  async function changeGroupReply(
    messageIds: string[],
    targetId: string | null,
  ): Promise<void> {
    await cancel();
    selectedMessageIds.value = messageIds;
    replyTargetId.value = targetId;
    candidates.value = [];
    generated.value = false;
    draft.value = "";
    selectedCandidate.value = -1;
  }
  async function copy(): Promise<void> {
    try {
      await bridge.copyReply(draft.value);
      notify("已复制，回到微信后由你发送");
    } catch (failure) {
      error.value = bridge.errorMessage(failure);
    }
  }
  async function togglePin(): Promise<void> {
    try {
      await bridge.windowAction(pinned.value ? "unpin" : "pin");
      pinned.value = !pinned.value;
    } catch (failure) {
      notify(bridge.errorMessage(failure));
    }
  }

  return {
    syncPreferences,
    historyOpen,
    backgroundEnabled,
    refreshingContext,
    contextReadAt,
    refreshContext,
    settingsTab,
    availableTones,
    chosenTone,
    isGroup,
    selectedMessageIds,
    replyTargetId,
    changeGroupReply,
    toneTemplates,
    selectedToneId,
    workspace,
    selectedId,
    contact,
    profile,
    activeItems,
    conflictCount,
    observationCount,
    profileOpen,
    profileTab,
    settingsOpen,
    contextOpen,
    task,
    tone,
    customTone,
    background,
    reference,
    useModel,
    manualTranscript,
    candidates,
    selectedCandidate,
    draft,
    generated,
    generatedMode,
    generating,
    mutationBusy,
    loading,
    pinned,
    toast,
    error,
    provider,
    notify,
    initialize,
    cancel,
    selectContact,
    toggleProfile,
    mutate,
    generate,
    chooseCandidate,
    copy,
    togglePin,
  };
});
