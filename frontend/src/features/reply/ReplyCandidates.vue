<script setup lang="ts">
import { ArrowUpRight, Check, Copy, Sparkles } from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
const store = useWorkspace();
</script>

<template>
  <section class="candidates-section" aria-live="polite">
    <div class="section-title">
      <strong>{{
        store.generated ? "可以这样回" : "让表达，恰到好处。"
      }}</strong><span v-if="store.generated" class="helper"
        >{{ store.generatedMode === "model" ? "模型生成" : "演示候选" }} · 3
        条</span
      >
    </div>
    <div v-if="!store.generated" class="reply-empty">
      <span class="empty-symbol"
        ><Sparkles :size="22" :stroke-width="1.3"
      /></span>
      <p>选好语气，剩下的交给回声。</p>
      <span>先给你三个思路，再改成你自己的话。</span>
    </div>
    <div v-else class="candidate-list">
      <button
        v-for="(candidate, index) in store.candidates"
        :key="index"
        class="candidate"
        :aria-pressed="store.selectedCandidate === index"
        @click="store.chooseCandidate(index)"
      >
        <span class="candidate-number">{{
          String(index + 1).padStart(2, "0")
        }}</span><span>{{ candidate }}</span
        ><Check
          v-if="store.selectedCandidate === index"
          :size="14"
        /><ArrowUpRight v-else :size="14" />
      </button>
    </div>
    <div v-if="store.selectedCandidate >= 0" class="draft-editor">
      <label for="reply-draft">改成你的话</label
      ><textarea
        id="reply-draft"
        v-model="store.draft"
        rows="3"
        maxlength="20000"
      ></textarea>
      <div class="form-actions">
        <span class="helper">由你确认，再回到微信发送</span
        ><button
          class="button primary compact"
          :disabled="!store.draft.trim()"
          @click="store.copy"
        >
          <Copy :size="13" />复制回复
        </button>
      </div>
    </div>
  </section>
</template>
