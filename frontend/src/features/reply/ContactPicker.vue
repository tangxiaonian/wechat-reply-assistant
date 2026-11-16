<script setup lang="ts">
import { ref, computed } from "vue";
import { Check, ChevronDown, ContactRound, Search, X } from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
const store = useWorkspace();
const open = ref(false);
const query = ref("");
const contacts = computed(
  () =>
    store.workspace?.contacts.filter((contact) =>
      `${contact.name}${contact.relationship}`.includes(query.value.trim()),
    ) ?? [],
);

async function select(id: string): Promise<void> {
  open.value = false;
  await store.selectContact(id);
}
</script>

<template>
  <div class="contact-picker">
    <button
      class="contact-current"
      aria-label="选择回复对象"
      :aria-expanded="open"
      @click="open = !open"
    >
      <span class="avatar" :class="store.contact?.tint">{{
        store.contact?.initial
      }}</span>
      <span class="contact-heading"
        ><span class="eyebrow">回复给</span
        ><strong>{{ store.contact?.name }}<ChevronDown :size="13" /></strong
      ></span>
    </button>
    <button
      v-if="!store.isGroup"
      class="about-button"
      @click="
        !store.profileOpen ? store.toggleProfile() : (store.profileTab = 'info')
      "
    >
      <ContactRound :size="15" />关于 TA<span
        v-if="store.conflictCount"
        class="attention-dot"
      ></span>
    </button>
    <div v-if="open" class="contact-menu">
      <div class="menu-title">
        <span>选择示例会话</span
        ><button
          class="icon-button"
          aria-label="关闭会话选择"
          @click="open = false"
        >
          <X :size="15" />
        </button>
      </div>
      <label class="search-field"
        ><Search :size="15" /><input
          v-model="query"
          placeholder="搜索姓名或关系"
          aria-label="搜索示例会话"
      /></label>
      <button
        v-for="contact in contacts"
        :key="contact.id"
        class="contact-option"
        @click="select(contact.id)"
      >
        <span class="avatar small" :class="contact.tint">{{
          contact.initial
        }}</span><span
          ><strong>{{ contact.name }}</strong
          ><small
            >{{ contact.relationship }} · {{ contact.description }}</small
          ></span
        ><Check v-if="store.selectedId === contact.id" :size="15" />
      </button>
      <p v-if="!contacts.length" class="empty-small">没有匹配的示例会话</p>
      <p class="menu-footnote">这里是虚构示例，未读取你的微信联系人。</p>
    </div>
  </div>
</template>
