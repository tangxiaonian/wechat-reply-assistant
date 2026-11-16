<script setup lang="ts">
import { computed } from "vue";
import { ChevronRight, MessagesSquare } from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
const store = useWorkspace();
const target = computed(
  () =>
    store.contact?.members?.find((entry) => entry.id === store.replyTargetId)
      ?.name ?? "整个群",
);
</script>
<template>
  <button class="conversation-summary" @click="store.historyOpen = true">
    <span class="conversation-summary-top"
      ><MessagesSquare :size="14" /><strong>聊天记录</strong
      ><span>{{
        store.manualTranscript
          ? "手动片段"
          : `示例 · ${store.contact?.messages.length ?? 0} 条`
      }}</span
      ><ChevronRight :size="14"
    /></span>
    <span v-if="store.isGroup" class="conversation-summary-text">{{
      store.selectedMessageIds.length
        ? `已选 ${store.selectedMessageIds.length} 条 · 回复${target}`
        : "选择你想回复的群消息"
    }}</span>
    <span v-else class="conversation-summary-text">{{
      store.manualTranscript
        ? "已导入片段，点击查看"
        : store.contact?.messages.at(-1)?.text
    }}</span>
  </button>
</template>
