<script setup lang="ts">
import { ref, computed } from "vue";
import { ArrowDown, Check, Quote, ScanText } from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
import type { Proposal } from "../../types/workspace";
const props = defineProps<{ kind: "observation" | "conflict" }>();
const store = useWorkspace();
const recentFor = ref<string | null>(null);
const expiry = ref("");
const proposals = computed(
  () =>
    store.profile?.proposals.filter((entry) => entry.kind === props.kind) ?? [],
);

async function decide(
  proposal: Proposal,
  decision: "accept" | "accept_as_recent" | "keep_existing" | "ignore",
): Promise<void> {
  try {
    await store.mutate({
      type: "decide",
      contactId: store.selectedId,
      proposalId: proposal.id,
      decision,
      expiresAtMs:
        decision === "accept_as_recent" && expiry.value
          ? new Date(`${expiry.value}T23:59:59`).getTime()
          : undefined,
    });
    recentFor.value = null;
  } catch {
    /* The store keeps the classified error visible in the reply panel. */
  }
}
</script>

<template>
  <div>
    <p v-if="kind === 'observation'" class="observation-explainer">
      <ScanText :size="15" /><span
        >从聊天中观察到的特点，确认后才用于回复。当前为示例建议。</span
      >
    </p>
    <div v-if="!proposals.length" class="profile-empty">
      <Check :size="22" :stroke-width="1.3" />
      <p>
        {{
          kind === "observation" ? "暂时没有待确认的观察" : "没有需要核对的变化"
        }}
      </p>
      <span>有依据的新发现会出现在这里。</span>
    </div>
    <article
      v-for="proposal in proposals"
      :key="proposal.id"
      class="proposal"
      :class="{ 'is-conflict': kind === 'conflict' }"
    >
      <div class="proposal-title">
        <span class="mini-label">{{
          kind === "conflict" ? "有不同信息 · 示例" : "待你确认 · 示例"
        }}</span><strong>{{ proposal.subjectLabel }}</strong>
      </div>
      <template v-if="kind === 'conflict'"
        ><div class="comparison">
          <span>你的资料 · 已锁定</span>
          <p>
            {{
              store.profile?.items.find(
                (item) => item.id === proposal.targetItemId,
              )?.content
            }}
          </p>
        </div>
        <ArrowDown :size="14" class="comparison-arrow" />
        <div class="comparison">
          <span>新聊天中的说法</span>
          <p>{{ proposal.content }}</p>
        </div></template
      >
      <p v-else class="observation-content">{{ proposal.content }}</p>
      <details class="source-details">
        <summary><Quote :size="11" />查看聊天依据</summary>
        <blockquote v-for="entry in proposal.evidence" :key="entry.messageId">
          “{{ entry.quote }}”<span>{{ entry.at }}</span>
        </blockquote>
      </details>
      <p v-if="kind === 'conflict'" class="helper conflict-hint">
        可能只是近期偏好。原资料已保留，等你决定。
      </p>
      <div class="proposal-actions">
        <button
          class="button compact"
          :disabled="store.mutationBusy"
          @click="
            decide(proposal, kind === 'conflict' ? 'keep_existing' : 'ignore')
          "
        >
          {{ kind === "conflict" ? "保留原资料" : "忽略这条" }}</button><button
          class="button compact"
          :class="{ primary: kind === 'observation' }"
          :disabled="store.mutationBusy"
          @click="decide(proposal, 'accept')"
        >
          {{ kind === "conflict" ? "采纳替换" : "确认并加入" }}</button
        ><button
          v-if="kind === 'conflict'"
          class="text-button"
          @click="recentFor = recentFor === proposal.id ? null : proposal.id"
        >
          加入近期偏好
        </button>
      </div>
      <form
        v-if="recentFor === proposal.id"
        class="recent-confirm"
        @submit.prevent="decide(proposal, 'accept_as_recent')"
      >
        <label
          >参考到哪一天<input v-model="expiry" type="date" required /></label
        ><button class="button primary compact" type="submit">确认加入</button>
      </form>
    </article>
  </div>
</template>
