<script setup lang="ts">
import { ref } from "vue";
import { ChevronDown, MessageCircle, PencilLine } from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
const store = useWorkspace();
const editing = ref(false);
const text = ref("");

/** Manual context remains transient and does not enter the long-term communication profile. */
async function apply(): Promise<void> {
  if (!text.value.trim()) {
    store.error = "请先填写对话，并标注“我：”或“对方：”";
    return;
  }
  if (text.value.length > 20_000) {
    store.error = "手动上下文最多 20,000 字符";
    return;
  }
  await store.cancel();
  store.manualTranscript = text.value.trim();
  store.reference = false;
  store.contextOpen = false;
  editing.value = false;
  store.generated = false;
  store.candidates = [];
  store.draft = "";
  store.notify("已改用手动片段，默认不带示例档案；连接模型后可据此生成");
}
</script>

<template>
  <section class="chat-context">
    <button
      class="context-summary"
      :aria-expanded="store.contextOpen"
      @click="store.contextOpen = !store.contextOpen"
    >
      <span
        ><MessageCircle :size="13" />{{
          store.manualTranscript ? "手动上下文" : "近期聊天 · 示例"
        }}<span class="context-count">{{
          store.manualTranscript ? "当前片段" : "3 条"
        }}</span></span
      ><ChevronDown :size="13" :class="{ rotated: store.contextOpen }" />
    </button>
    <div v-if="!store.contextOpen" class="latest-message">
      <span>对方</span>
      <p>
        {{
          store.manualTranscript
            ? "已提供手动片段，展开查看"
            : store.contact?.messages.at(-1)?.text
        }}
      </p>
    </div>
    <div v-else class="context-content">
      <div v-if="store.manualTranscript" class="manual-preview">
        {{ store.manualTranscript }}
      </div>
      <div v-else class="chat-messages">
        <div
          v-for="message in store.contact?.messages"
          :key="message.id"
          class="chat-message"
          :class="{ mine: message.direction === 'self' }"
        >
          <div class="message-meta">
            {{ message.direction === "self" ? "我" : store.contact?.name
            }}<span>{{ message.at }}</span>
          </div>
          <p>{{ message.text }}</p>
        </div>
      </div>
      <div class="context-actions">
        <button class="text-button" @click="editing = !editing">
          <PencilLine :size="12" />手动提供上下文</button
        ><button
          v-if="store.manualTranscript"
          class="text-button"
          @click="store.manualTranscript = null"
        >
          返回示例
        </button>
      </div>
      <form v-if="editing" class="manual-form" @submit.prevent="apply">
        <textarea
          v-model="text"
          rows="5"
          maxlength="20000"
          aria-label="手动聊天上下文"
          placeholder="对方：周六两点见？&#10;我：我可能要晚一点。&#10;对方：那两点半呢？"
        ></textarea>
        <div class="form-actions">
          <span class="helper">标明“我 / 对方”，仅用于本次回复</span
          ><button class="button compact" type="submit">使用片段</button>
        </div>
      </form>
    </div>
  </section>
</template>
