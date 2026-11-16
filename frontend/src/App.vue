<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { native } from "./bridge";
import { useWorkspace } from "./stores/workspace";
import WindowChrome from "./components/WindowChrome.vue";
import ReplyPanel from "./features/reply/ReplyPanel.vue";
import ProfileDrawer from "./features/profile/ProfileDrawer.vue";
import SettingsPanel from "./features/settings/SettingsPanel.vue";
import ChatHistoryPanel from "./features/history/ChatHistoryPanel.vue";
import { useAutoSync } from "./composables/useAutoSync";

const store = useWorkspace();
useAutoSync();
/** Keeps the primary keyboard action scoped to this application. */
function onKey(event: KeyboardEvent): void {
  if (
    (event.metaKey || event.ctrlKey) &&
    event.key === "Enter" &&
    !store.settingsOpen &&
    !store.historyOpen
  ) {
    event.preventDefault();
    void store.generate();
  }
  if (event.key === "Escape") {
    store.settingsOpen = false;
    store.historyOpen = false;
  }
}
onMounted(() => {
  void store.initialize();
  window.addEventListener("keydown", onKey);
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKey);
  void store.cancel();
});
</script>

<template>
  <div class="desktop" :class="{ 'is-native': native }">
    <div v-if="!native" class="preview-caption">
      <span>回声 · macOS 体验预览</span><span>v0.1 / 示例数据</span>
    </div>
    <div
      class="app-window"
      :class="{ 'profile-closed': !store.profileOpen || store.isGroup }"
    >
      <WindowChrome />
      <div v-if="store.loading" class="loading-screen">
        <span class="spinner"></span>正在打开回声…
      </div>
      <div v-else-if="!store.workspace" class="loading-screen">
        <p role="alert">{{ store.error }}</p>
        <button class="button" @click="store.initialize">重新载入</button>
      </div>
      <div v-else class="workspace">
        <ProfileDrawer v-if="store.profileOpen && !store.isGroup" />
        <ReplyPanel />
      </div>
      <ChatHistoryPanel v-if="store.historyOpen" />
      <SettingsPanel v-if="store.settingsOpen" />
      <div v-if="store.toast" class="toast" role="status" aria-live="polite">
        {{ store.toast }}
      </div>
    </div>
    <div v-if="!native" class="preview-bottom">
      <span>浏览器中体验页面与档案交互 · 模型连接和快捷键请使用桌面版</span
      ><kbd>⌘ ↵ 生成</kbd>
    </div>
  </div>
</template>
