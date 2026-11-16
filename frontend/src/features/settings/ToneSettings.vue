<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Pencil, Plus, Trash2 } from "@lucide/vue";
import {
  loadToneTemplates,
  saveToneTemplate,
  deleteToneTemplate,
  errorMessage,
} from "../../bridge";
import { useWorkspace } from "../../stores/workspace";
import type { ToneTemplate } from "../../types/workspace";
import ToneEditor from "./ToneEditor.vue";
const store = useWorkspace();
const editing = ref<ToneTemplate | null>(null);
const editorOpen = ref(false);
const busy = ref(false);
const error = ref("");
onMounted(async () => {
  try {
    store.toneTemplates = await loadToneTemplates();
  } catch (failure) {
    error.value = errorMessage(failure);
  }
});
/** Settings owns persistence; the chat panel only consumes enabled templates. */
async function save(template: ToneTemplate, close = true): Promise<void> {
  busy.value = true;
  error.value = "";
  try {
    store.toneTemplates = await saveToneTemplate(template);
    if (close) {
      editorOpen.value = false;
    }
    store.notify(
      template.enabled
        ? "语气已保存，聊天面板中可选"
        : "语气已关闭，聊天面板中隐藏",
    );
  } catch (failure) {
    error.value = errorMessage(failure);
  } finally {
    busy.value = false;
  }
}
async function remove(id: string): Promise<void> {
  busy.value = true;
  try {
    store.toneTemplates = await deleteToneTemplate(id);
    store.notify("已删除自定义语气");
  } catch (failure) {
    error.value = errorMessage(failure);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="tone-management" aria-label="语气管理">
    <p class="tone-intro">
      把你平时说话的感觉存下来。<span
        >在这里新增和管理，开启的语气才出现在聊天面板。</span
      >
    </p>
    <button
      class="button primary"
      @click="
        editing = null;
        editorOpen = true;
      "
    >
      <Plus :size="14" />新增语气
    </button>
    <ToneEditor
      v-if="editorOpen"
      :key="editing?.id ?? 'new'"
      :template="editing"
      :busy="busy"
      @close="editorOpen = false"
      @save="save"
    />
    <p v-if="error" class="error-text" role="alert">{{ error }}</p>
    <article
      v-for="tone in store.toneTemplates"
      :key="tone.id"
      class="tone-setting-row"
    >
      <div>
        <strong>{{ tone.name }}</strong><span class="helper">{{
          tone.baseTone ? "内置" : "自定义"
        }}</span
        ><button
          class="icon-button"
          :aria-label="`编辑语气${tone.name}`"
          @click="
            editing = { ...tone };
            editorOpen = true;
          "
        >
          <Pencil :size="12" /></button
        ><button
          v-if="!tone.baseTone"
          class="icon-button"
          :aria-label="`删除语气${tone.name}`"
          :disabled="busy"
          @click="remove(tone.id)"
        >
          <Trash2 :size="12" /></button
        ><label class="tone-switch"
          ><input
            type="checkbox"
            :checked="tone.enabled"
            :disabled="busy"
            :aria-label="`开启语气${tone.name}`"
            @change="
              save(
                {
                  ...tone,
                  enabled: ($event.target as HTMLInputElement).checked,
                },
                false,
              )
            "
          /><span>{{ tone.enabled ? "已开启" : "已关闭" }}</span></label
        >
      </div>
      <p>{{ tone.instruction }}</p>
    </article>
    <p class="helper">
      模板在模型生成时使用；演示模式仍是预设候选。临时调整在聊天面板的“本次补充”填写。
    </p>
  </section>
</template>
