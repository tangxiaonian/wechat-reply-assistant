<script setup lang="ts">
import { ArrowLeft, RefreshCw } from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
import ChatContext from "../reply/ChatContext.vue";
import GroupContext from "../reply/GroupContext.vue";
const store = useWorkspace();
store.contextOpen = true;
</script>
<template>
  <section
    class="settings-panel history-panel"
    role="dialog"
    aria-modal="true"
    aria-labelledby="history-title"
  >
    <header>
      <div>
        <span class="eyebrow">{{ store.contact?.name }}</span>
        <h2 id="history-title">聊天记录</h2>
      </div>
      <button class="text-button" @click="store.historyOpen = false">
        <ArrowLeft :size="13" />返回回复
      </button>
    </header>
    <div class="history-toolbar">
      <span class="helper">{{
        store.manualTranscript
          ? "手动片段"
          : `示例载入于 ${new Date(store.contextReadAt).toLocaleTimeString("zh-CN")}`
      }}</span><button
        class="button compact"
        :disabled="store.refreshingContext || !!store.manualTranscript"
        @click="store.refreshContext()"
      >
        <RefreshCw :size="12" :class="{ spin: store.refreshingContext }" />{{
          store.refreshingContext ? "载入中…" : "刷新示例"
        }}
      </button>
    </div>
    <div class="settings-scroll">
      <p v-if="store.syncPreferences.enabled" class="helper">
        自动刷新示例 · 每 {{ store.syncPreferences.intervalSeconds }} 秒
      </p>
      <p class="history-source-note">
        当前为虚构示例。刷新会重新载入示例，尚不能获取微信最新消息。
      </p>
      <GroupContext v-if="store.isGroup" /><ChatContext v-else />
    </div>
    <footer class="settings-footer">
      <span>{{
        store.isGroup
          ? `已选 ${store.selectedMessageIds.length} 条`
          : "仅作本次回复上下文"
      }}</span><button class="text-button" @click="store.historyOpen = false">
        返回生成回复<ArrowLeft :size="12" />
      </button>
    </footer>
  </section>
</template>
