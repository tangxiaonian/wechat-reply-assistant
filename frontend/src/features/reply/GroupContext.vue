<script setup lang="ts">
import { computed } from "vue";
import { MessageSquareQuote, UsersRound } from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
const store = useWorkspace();
const recipients = computed(
  () => store.contact?.members?.filter((member) => member.id !== "self") ?? [],
);

/** Checkbox selection keeps stable message IDs and never creates member profiles. */
function toggle(id: string): void {
  const ids = store.selectedMessageIds.includes(id)
    ? store.selectedMessageIds.filter((entry) => entry !== id)
    : [...store.selectedMessageIds, id];
  void store.changeGroupReply(ids, store.replyTargetId);
}
function target(event: Event): void {
  const value = (event.target as HTMLSelectElement).value;
  void store.changeGroupReply([...store.selectedMessageIds], value || null);
}
</script>

<template>
  <section class="group-context" aria-label="群聊消息选择">
    <div class="group-context-title">
      <UsersRound :size="14" /><strong>群里正在聊</strong
      ><span>虚构示例 · {{ store.contact?.messages.length }} 条</span>
    </div>
    <p class="helper">选中你想接的话，其他消息只作为上下文。</p>
    <div class="group-messages">
      <label
        v-for="message in store.contact?.messages"
        :key="message.id"
        class="group-message"
        :class="{
          selected: store.selectedMessageIds.includes(message.id),
          'group-mine': message.direction === 'self',
        }"
      >
        <input
          v-if="message.direction !== 'self'"
          type="checkbox"
          :checked="store.selectedMessageIds.includes(message.id)"
          :aria-label="`回复${message.senderName}：${message.text}`"
          @change="toggle(message.id)"
        />
        <span class="group-message-body"
          ><span class="message-meta"
            >{{ message.direction === "self" ? "我" : message.senderName
            }}<span>{{ message.at }}</span></span
          ><span v-if="message.replyToId" class="group-quote"
            ><MessageSquareQuote :size="11" />{{
              store.contact?.messages.find(
                (entry) => entry.id === message.replyToId,
              )?.text
            }}</span
          ><span class="group-message-text">{{ message.text }}</span></span
        >
      </label>
    </div>
    <div class="group-selection">
      <span>{{
        store.selectedMessageIds.length
          ? `已选 ${store.selectedMessageIds.length} 条`
          : "先选一条或多条消息"
      }}</span><label
        >回复给<select
          :value="store.replyTargetId ?? ''"
          aria-label="群聊回复对象"
          @change="target"
        >
          <option value="">整个群</option>
          <option
            v-for="member in recipients"
            :key="member.id"
            :value="member.id"
          >
            {{ member.name }}
          </option>
        </select></label
      >
    </div>
    <p class="helper group-no-profile">
      只参考本次群聊，不创建群或成员画像。文字 @ 需回微信后手动选择联系人。
    </p>
  </section>
</template>
