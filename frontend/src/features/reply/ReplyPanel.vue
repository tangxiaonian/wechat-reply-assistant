<script setup lang="ts">
import { computed } from "vue";
import {
  ArrowRight,
  BookOpen,
  Info,
  LoaderCircle,
  Sparkles,
  X,
} from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
import ContactPicker from "./ContactPicker.vue";
import ConversationSummary from "./ConversationSummary.vue";
import BackgroundField from "./BackgroundField.vue";
import TonePicker from "./TonePicker.vue";
import ReplyCandidates from "./ReplyCandidates.vue";
const store = useWorkspace();
const referenceSummary = computed(() =>
  store.activeItems.slice(0, 2).map((item) => item.subjectLabel),
);
const tasks = [
  { value: "reply" as const, label: "回复对方" },
  { value: "rewrite" as const, label: "改写我的话" },
  { value: "follow_up" as const, label: "主动跟进" },
];
</script>

<template>
  <main class="reply-panel">
    <div class="main-scroll">
      <div class="reply-heading">
        <span class="eyebrow">你的聊天小助手</span
        ><span class="mode-pill"
          ><span></span>{{ store.isGroup ? "群聊示例" : "私聊示例" }}</span
        >
      </div>
      <ContactPicker />
      <ConversationSummary />
      <div v-if="!store.isGroup" class="profile-reference">
        <label class="check-label"
          ><input v-model="store.reference" type="checkbox" /><BookOpen
            :size="13"
          />本次参考沟通档案<span class="count-label">{{
            store.activeItems.length
          }}</span></label
        >
        <div v-if="store.reference" class="reference-tags">
          <span v-for="label in referenceSummary" :key="label">{{
            label
          }}</span><button
            class="text-button"
            @click="
              !store.profileOpen
                ? store.toggleProfile()
                : (store.profileTab = 'info')
            "
          >
            查看档案<ArrowRight :size="11" />
          </button>
        </div>
      </div>
      <div
        class="task-switch"
        :class="{ 'group-task': store.isGroup }"
        role="group"
        aria-label="本次任务"
      >
        <button
          v-for="task in tasks"
          v-show="!store.isGroup || task.value !== 'rewrite'"
          :key="task.value"
          :aria-pressed="store.task === task.value"
          @click="store.task = task.value"
        >
          {{
            store.isGroup
              ? task.value === "reply"
                ? "回复选中消息"
                : "在群里补充"
              : task.label
          }}
        </button>
      </div>
      <TonePicker />
      <BackgroundField />
      <div class="generation-controls">
        <label class="model-mode"
          ><input
            v-model="store.useModel"
            type="checkbox"
          />使用已连接模型</label
        ><button
          v-if="!store.provider.configured"
          class="text-button"
          @click="store.settingsOpen = true"
        >
          连接模型<ArrowRight :size="11" /></button
        ><span v-else class="helper">{{ store.provider.model }}</span>
      </div>
      <button
        v-if="!store.generating"
        class="generate-button"
        @click="store.generate"
      >
        <Sparkles :size="16" />{{
          store.generated ? "再帮我想一组" : "帮我想怎么回"
        }}<kbd>⌘ ↵</kbd>
      </button>
      <button v-else class="generate-button generating" @click="store.cancel">
        <LoaderCircle :size="16" class="spin" />正在想怎么回…<span>取消</span>
      </button>
      <div v-if="store.error" class="inline-error" role="alert">
        <Info :size="14" /><span>{{ store.error }}</span
        ><button
          class="icon-button"
          aria-label="关闭错误提示"
          @click="store.error = ''"
        >
          <X :size="13" />
        </button>
      </div>
      <ReplyCandidates />
      <p
        v-if="store.generated && store.generatedMode === 'demo'"
        class="demo-note"
      >
        <Info
          :size="12"
        />演示展示预设场景；自定义语气和任意背景需连接模型后处理。
      </p>
    </div>
    <footer class="panel-footer">
      <span><span class="status-dot"></span>候选可编辑 · 由你发送</span
      ><span>⌥ Space 唤起</span>
    </footer>
  </main>
</template>
