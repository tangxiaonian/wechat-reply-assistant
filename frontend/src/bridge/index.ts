import { invoke, isTauri } from "@tauri-apps/api/core";
import fixture from "../../../shared/demo-workspace.json";
import { applyProfileAction } from "../domain/profiles";
import type {
  ProfileAction,
  ProviderInput,
  ProviderSummary,
  ReplyInput,
  ReplyResult,
  Workspace,
  ToneTemplate,
  SyncSettings,
} from "../types/workspace";
import defaultTones from "../../../shared/tone-defaults.json";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** Sync preferences contain no messages and can be restored independently of reply drafts. */
export async function syncSettings(
  input?: SyncSettings,
): Promise<SyncSettings> {
  if (
    input &&
    (!Number.isInteger(input.intervalSeconds) ||
      input.intervalSeconds < 5 ||
      input.intervalSeconds > 300)
  ) {
    throw new Error("同步间隔需要是 5～300 秒的整数");
  }
  if (native) {
    return invoke("sync_settings", { input: input ?? null });
  }
  if (input) {
    localStorage.setItem("echo-sync-settings-v1", JSON.stringify(input));
    return input;
  }
  const saved = localStorage.getItem("echo-sync-settings-v1");
  return saved
    ? (JSON.parse(saved) as SyncSettings)
    : { enabled: false, intervalSeconds: 30 };
}
export async function windowVisible(): Promise<boolean> {
  return native
    ? getCurrentWindow().isVisible()
    : document.visibilityState !== "hidden";
}

/** Tone preferences never contain chat context and only enabled entries are offered in chat. */
export async function loadToneTemplates(): Promise<ToneTemplate[]> {
  if (native) {
    return invoke("tone_templates", { input: null });
  }
  const saved = localStorage.getItem("echo-tone-templates-v1");
  return saved
    ? (JSON.parse(saved) as ToneTemplate[])
    : (structuredClone(defaultTones) as ToneTemplate[]);
}
export async function saveToneTemplate(
  template: ToneTemplate,
): Promise<ToneTemplate[]> {
  if (
    !template.name.trim() ||
    template.name.length > 20 ||
    !template.instruction.trim() ||
    template.instruction.length > 300
  ) {
    throw new Error("语气名称限 1～20 字，描述限 1～300 字");
  }
  if (native) {
    return invoke("tone_templates", { input: { type: "save", template } });
  }
  const list = await loadToneTemplates();
  const result = [
    ...list.filter((entry) => entry.id !== template.id),
    template,
  ];
  localStorage.setItem("echo-tone-templates-v1", JSON.stringify(result));
  return result;
}
export async function deleteToneTemplate(id: string): Promise<ToneTemplate[]> {
  if (native) {
    return invoke("tone_templates", { input: { type: "delete", id } });
  }
  const result = (await loadToneTemplates()).filter((entry) => entry.id !== id);
  localStorage.setItem("echo-tone-templates-v1", JSON.stringify(result));
  return result;
}

export const native = isTauri();
const browserStorageKey = "echo-preview-profiles-v1";

/** Reads browser preview data without ever persisting chat messages or generated drafts. */
export async function loadWorkspace(): Promise<Workspace> {
  if (native) {
    return invoke<Workspace>("load_workspace");
  }
  const workspace = structuredClone(fixture) as Workspace;
  const saved = localStorage.getItem(browserStorageKey);
  if (saved) {
    try {
      const profiles: unknown = JSON.parse(saved);
      if (
        profiles &&
        typeof profiles === "object" &&
        Object.keys(workspace.profiles).every((id) => id in profiles)
      ) {
        workspace.profiles = profiles as Workspace["profiles"];
      }
    } catch {
      localStorage.removeItem(browserStorageKey);
    }
  }
  return workspace;
}

/** Persists profile operations through the native service, or the isolated browser preview. */
export async function profileAction(input: ProfileAction): Promise<Workspace> {
  if (native) {
    return invoke<Workspace>("profile_action", { input });
  }
  const workspace = await loadWorkspace();
  const profile = workspace.profiles[input.contactId];
  if (!profile) {
    throw new Error("没有找到这个联系人的档案");
  }
  workspace.profiles[input.contactId] = applyProfileAction(profile, input);
  localStorage.setItem(browserStorageKey, JSON.stringify(workspace.profiles));
  return workspace;
}

export async function copyReply(text: string): Promise<void> {
  if (native) {
    return invoke("copy_reply", { text });
  }
  await navigator.clipboard.writeText(text);
}

export async function windowAction(
  action: "hide" | "minimize" | "expand" | "collapse" | "pin" | "unpin",
): Promise<void> {
  if (native) {
    await invoke("window_action", { action });
  }
}

export async function getProvider(): Promise<ProviderSummary> {
  if (native) {
    return invoke("get_provider");
  }
  return {
    name: "",
    baseUrl: "",
    model: "",
    configured: false,
    consent: false,
  };
}

/** Secrets only cross the native bridge on explicit save, and are never stored by this module. */
export async function saveProvider(
  input: ProviderInput,
): Promise<ProviderSummary> {
  if (!native) {
    throw new Error("请在 macOS 桌面版配置模型，API Key 仅保存到系统钥匙串");
  }
  return invoke("save_provider", { input });
}

export async function removeProvider(): Promise<void> {
  if (native) {
    await invoke("remove_provider");
  }
}

export async function testProvider(): Promise<number> {
  if (!native) {
    throw new Error("模型连接需在 macOS 桌面版测试");
  }
  return invoke("test_provider");
}

export async function generateReply(input: ReplyInput): Promise<ReplyResult> {
  return invoke("generate_reply", { input });
}
export async function cancelReply(requestId: string): Promise<void> {
  if (native) {
    await invoke("cancel_reply", { requestId });
  }
}

/** Converts native structured errors to a user message without leaking private diagnostics. */
export function errorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (
    typeof error === "object" &&
    error &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }
  return typeof error === "string" ? error : "操作失败，请重试";
}
