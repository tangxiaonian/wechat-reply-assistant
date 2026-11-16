<script setup lang="ts">
import { Clock3 } from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
import ProfileProposals from "./ProfileProposals.vue";
const store = useWorkspace();
</script>

<template>
  <div class="profile-history">
    <ProfileProposals kind="conflict" />
    <div v-if="store.profile?.events.length" class="history-heading">
      <Clock3 :size="12" />最近变化
    </div>
    <article
      v-for="event in store.profile?.events"
      :key="event.id"
      class="history-event"
    >
      <span class="history-dot"></span
      ><time>{{
        new Date(event.createdAtMs).toLocaleString("zh-CN", {
          month: "numeric",
          day: "numeric",
          hour: "2-digit",
          minute: "2-digit",
        })
      }}</time>
      <p>{{ event.summary }}</p>
      <div v-if="event.before" class="history-before">{{ event.before }}</div>
      <div v-if="event.after" class="history-after">{{ event.after }}</div>
    </article>
  </div>
</template>
