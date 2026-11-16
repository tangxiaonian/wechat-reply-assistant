<script setup lang="ts">
import { computed } from "vue";
import { ChevronDown, LockKeyhole, Pencil, Quote, Timer } from "@lucide/vue";
import { categories, type ProfileItem } from "../../types/workspace";
const props = defineProps<{ item: ProfileItem }>();
defineEmits<{ edit: [item: ProfileItem] }>();
const expired = computed(
  () => props.item.expiresAtMs !== null && props.item.expiresAtMs <= Date.now(),
);
const source = computed(() =>
  props.item.sourceKind === "manual"
    ? "你补充"
    : props.item.sourceKind === "user_confirmed"
      ? "你确认"
      : "聊天事实 · 示例",
);
</script>

<template>
  <article class="profile-item" :class="{ expired }">
    <div class="profile-item-heading">
      <strong>{{ categories[item.category] }}</strong><span class="source-label"
        ><LockKeyhole v-if="item.locked" :size="10" />{{ source }}</span
      ><button
        class="icon-button subtle"
        :aria-label="`编辑${categories[item.category]}：${item.content}`"
        @click="$emit('edit', item)"
      >
        <Pencil :size="12" />
      </button>
    </div>
    <p>{{ item.content }}</p>
    <span v-if="item.expiresAtMs" class="expiry-label"
      ><Timer :size="11" />{{
        expired
          ? "已到期，不再参考"
          : `有效至 ${new Date(item.expiresAtMs).toLocaleDateString("zh-CN")}`
      }}</span
    >
    <details
      v-if="item.evidence.length && item.sourceKind !== 'manual'"
      class="source-details"
    >
      <summary><Quote :size="11" />查看来源<ChevronDown :size="11" /></summary>
      <blockquote v-for="entry in item.evidence" :key="entry.messageId">
        “{{ entry.quote }}”<span>{{ entry.at }}</span>
      </blockquote>
    </details>
  </article>
</template>
