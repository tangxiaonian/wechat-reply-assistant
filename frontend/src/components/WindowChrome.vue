<script setup lang="ts">
import {
  AudioLines,
  PanelLeftClose,
  PanelLeftOpen,
  Pin,
  Settings2,
} from "@lucide/vue";
import { native, windowAction } from "../bridge";
import { useWorkspace } from "../stores/workspace";
const store = useWorkspace();

/** Hiding ends the current editing session without modifying any WeChat window. */
async function close(): Promise<void> {
  await store.cancel();
  store.background = "";
  store.draft = "";
  store.manualTranscript = null;
  if (native) {
    await windowAction("hide");
  } else {
    store.notify("桌面版关闭后驻留托盘，可用 Option + Space 唤起");
  }
}
</script>

<template>
  <header class="window-chrome" data-tauri-drag-region>
    <div class="traffic-lights">
      <button
        class="traffic close"
        aria-label="隐藏窗口"
        @click="close"
      ></button>
      <button
        class="traffic minimize"
        aria-label="最小化窗口"
        @click="windowAction('minimize')"
      ></button>
      <button
        class="traffic expand"
        :disabled="store.isGroup"
        :aria-label="store.profileOpen ? '收起沟通档案' : '展开沟通档案'"
        @click="store.toggleProfile"
      ></button>
    </div>
    <div class="window-title" data-tauri-drag-region>
      <AudioLines :size="14" /><span>回声</span
      ><span class="edition">体验版</span>
    </div>
    <div class="chrome-actions">
      <button
        v-if="!store.isGroup"
        class="icon-button"
        :aria-label="store.profileOpen ? '收起档案' : '展开档案'"
        :aria-pressed="store.profileOpen"
        @click="store.toggleProfile"
      >
        <component
          :is="store.profileOpen ? PanelLeftClose : PanelLeftOpen"
          :size="15"
        />
      </button>
      <button
        class="icon-button"
        :class="{ active: store.pinned }"
        aria-label="置顶浮窗"
        :aria-pressed="store.pinned"
        @click="store.togglePin"
      >
        <Pin :size="14" />
      </button>
      <button
        class="icon-button"
        aria-label="打开设置"
        @click="store.settingsOpen = true"
      >
        <Settings2 :size="15" />
      </button>
    </div>
  </header>
</template>
