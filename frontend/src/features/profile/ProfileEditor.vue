<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { LockKeyhole, Trash2, X } from "@lucide/vue";
import {
  categories,
  type Category,
  type ProfileItem,
} from "../../types/workspace";
import { useWorkspace } from "../../stores/workspace";
const props = defineProps<{ item: ProfileItem | null }>();
const emit = defineEmits<{ close: [] }>();
const store = useWorkspace();
const category = ref<Category>(props.item?.category ?? "communication");
const content = ref(props.item?.content ?? "");
const locked = ref(props.item?.locked ?? true);
const expires = ref(
  props.item?.expiresAtMs
    ? new Date(props.item.expiresAtMs).toISOString().slice(0, 10)
    : "",
);
const exclude = ref(false);
const error = ref("");
const input = ref<HTMLTextAreaElement>();
const recent = computed(
  () => category.value === "recent" || props.item?.scope === "recent",
);
onMounted(() => {
  input.value?.focus();
});

/** Saves only this contact's profile; the current reply background is never copied here. */
async function save(): Promise<void> {
  error.value = "";
  const item: ProfileItem = {
    id: props.item?.id ?? crypto.randomUUID(),
    category: category.value,
    subjectKey: props.item?.subjectKey ?? `custom:${crypto.randomUUID()}`,
    subjectLabel: props.item?.subjectLabel ?? categories[category.value],
    content: content.value,
    sourceKind: "manual",
    locked: locked.value,
    scope: recent.value ? "recent" : "long_term",
    expiresAtMs:
      recent.value && expires.value
        ? new Date(`${expires.value}T23:59:59`).getTime()
        : null,
    revision: props.item?.revision ?? 0,
    evidence: [],
  };
  try {
    await store.mutate({ type: "save", contactId: store.selectedId, item });
    emit("close");
  } catch (failure) {
    error.value =
      failure instanceof Error ? failure.message : "保存失败，请检查内容";
  }
}
async function remove(): Promise<void> {
  if (!props.item) {
    return;
  }
  try {
    await store.mutate({
      type: "delete",
      contactId: store.selectedId,
      itemId: props.item.id,
      excludeFuture: exclude.value,
    });
    emit("close");
  } catch {
    error.value = "删除失败，请重试";
  }
}
</script>

<template>
  <div
    class="modal-backdrop"
    @click.self="emit('close')"
    @keydown.esc="emit('close')"
  >
    <section
      class="editor-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="profile-editor-title"
    >
      <header>
        <div>
          <span class="eyebrow">关于 {{ store.contact?.name }}</span>
          <h2 id="profile-editor-title">
            {{ item ? "编辑这条资料" : "补充关于 TA 的信息" }}
          </h2>
        </div>
        <button
          class="icon-button"
          aria-label="关闭资料编辑"
          @click="emit('close')"
        >
          <X :size="17" />
        </button>
      </header>
      <form @submit.prevent="save">
        <label
          >归入<select v-model="category">
            <option
              v-for="(name, value) in categories"
              :key="value"
              :value="value"
            >
              {{ name }}
            </option>
          </select></label
        ><label
          >你知道的事情<textarea
            ref="input"
            v-model="content"
            rows="4"
            maxlength="1000"
            required
            placeholder="例如：工作日午休不方便接电话"
          ></textarea></label
        ><label v-if="recent"
          >有效至<input v-model="expires" type="date" required /></label
        ><label class="check-label lock-option"
          ><input v-model="locked" type="checkbox" /><LockKeyhole
            :size="13"
          />锁定这条，AI 更新时先让我核对</label
        >
        <p class="helper">你的补充优先于 AI 观察，仅用于这个联系人。</p>
        <p v-if="error" class="error-text" role="alert">{{ error }}</p>
        <div class="editor-save">
          <button class="button" type="button" @click="emit('close')">
            取消</button
          ><button
            class="button primary"
            :disabled="store.mutationBusy"
            type="submit"
          >
            {{ store.mutationBusy ? "保存中…" : "保存到档案" }}
          </button>
        </div>
      </form>
      <div v-if="item" class="editor-delete">
        <label class="check-label"
          ><input v-model="exclude" type="checkbox" />以后不再记录此项</label
        ><button
          class="text-button danger"
          :disabled="store.mutationBusy"
          @click="remove"
        >
          <Trash2 :size="12" />删除资料
        </button>
        <p class="helper">同时清理关联来源与正文历史。</p>
      </div>
    </section>
  </div>
</template>
