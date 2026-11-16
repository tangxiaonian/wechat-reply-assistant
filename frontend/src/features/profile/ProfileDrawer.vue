<script setup lang="ts">
import { ref, watch, toRaw } from "vue";
import {
  ArrowRight,
  ChevronRight,
  LockKeyhole,
  Plus,
  RefreshCw,
  ShieldCheck,
  X,
} from "@lucide/vue";
import { useWorkspace } from "../../stores/workspace";
import type { ProfileItem, ProfileTab } from "../../types/workspace";
import ProfileItemCard from "./ProfileItemCard.vue";
import ProfileEditor from "./ProfileEditor.vue";
import ProfileProposals from "./ProfileProposals.vue";
import ProfileHistory from "./ProfileHistory.vue";
const store = useWorkspace();
const editorOpen = ref(false);
const editing = ref<ProfileItem | null>(null);
const tabs: { value: ProfileTab; label: string }[] = [
  { value: "info", label: "资料" },
  { value: "observations", label: "AI 观察" },
  { value: "history", label: "更新记录" },
];
watch(
  () => store.selectedId,
  () => {
    editorOpen.value = false;
  },
);
function edit(item: ProfileItem | null): void {
  editing.value = item ? structuredClone(toRaw(item)) : null;
  editorOpen.value = true;
}
async function demoUpdate(): Promise<void> {
  try {
    await store.mutate({ type: "demo_update", contactId: store.selectedId });
    store.profileTab = "history";
  } catch {
    /* The store displays the error. */
  }
}
</script>

<template>
  <aside class="profile-drawer" aria-label="关于 TA 的沟通档案">
    <header class="drawer-header">
      <div>
        <span class="eyebrow">关于 TA</span>
        <h1>更了解，才更会聊。</h1>
      </div>
      <button
        class="icon-button"
        aria-label="关闭沟通档案"
        @click="store.toggleProfile"
      >
        <X :size="16" />
      </button>
    </header>
    <div class="profile-person">
      <span class="avatar" :class="store.contact?.tint">{{
        store.contact?.initial
      }}</span>
      <div>
        <strong>{{ store.contact?.name }}</strong><span
          >{{ store.contact?.relationship }}<b>·</b>独立沟通档案</span
        >
      </div>
      <span class="sample-badge">示例</span>
    </div>
    <div class="profile-tabs" role="tablist" aria-label="沟通档案">
      <button
        v-for="tab in tabs"
        :id="`tab-${tab.value}`"
        :key="tab.value"
        role="tab"
        :aria-selected="store.profileTab === tab.value"
        :aria-controls="`pane-${tab.value}`"
        @click="store.profileTab = tab.value"
      >
        {{ tab.label
        }}<span
          v-if="tab.value === 'observations' && store.observationCount"
          class="tab-count"
          >{{ store.observationCount }}</span
        ><span
          v-if="tab.value === 'history' && store.conflictCount"
          class="tab-count amber"
          >{{ store.conflictCount }}</span
        >
      </button>
    </div>
    <div class="profile-scroll">
      <section
        id="pane-info"
        role="tabpanel"
        aria-labelledby="tab-info"
        :hidden="store.profileTab !== 'info'"
      >
        <button
          v-if="store.conflictCount"
          class="conflict-banner"
          @click="store.profileTab = 'history'"
        >
          <span class="banner-icon"><RefreshCw :size="14" /></span
          ><span
            ><strong>发现 {{ store.conflictCount }} 条不同的说法</strong
            ><small>原资料保留，等你核对</small></span
          ><ChevronRight :size="14" />
        </button>
        <div class="profile-section-label">
          <span>已确认的资料</span
          ><span>{{ store.profile?.items.length }} 条</span>
        </div>
        <ProfileItemCard
          v-for="item in store.profile?.items"
          :key="item.id"
          :item="item"
          @edit="edit"
        />
        <p v-if="!store.profile?.items.length" class="empty-small">
          还没有资料，先补充一件你知道的事。
        </p>
        <button class="add-profile" @click="edit(null)">
          <Plus :size="15" />补充关于 TA 的信息
        </button>
        <details v-if="store.profile?.exclusions.length" class="exclusions">
          <summary>
            已排除 {{ store.profile.exclusions.length }} 项<ChevronRight
              :size="12"
            />
          </summary>
          <div
            v-for="exclusion in store.profile.exclusions"
            :key="exclusion.id"
          >
            <span>{{ exclusion.subjectLabel }}</span><button
              class="text-button"
              @click="
                store.mutate({
                  type: 'remove_exclusion',
                  contactId: store.selectedId,
                  exclusionId: exclusion.id,
                })
              "
            >
              解除排除
            </button>
          </div>
        </details>
      </section>
      <section
        id="pane-observations"
        role="tabpanel"
        aria-labelledby="tab-observations"
        :hidden="store.profileTab !== 'observations'"
      >
        <ProfileProposals kind="observation" />
      </section>
      <section
        id="pane-history"
        role="tabpanel"
        aria-labelledby="tab-history"
        :hidden="store.profileTab !== 'history'"
      >
        <ProfileHistory />
      </section>
    </div>
    <footer class="profile-footer">
      <div>
        <span><ShieldCheck :size="13" />资料保存在本机</span
        ><button
          class="text-button"
          :disabled="store.mutationBusy"
          @click="demoUpdate"
        >
          <RefreshCw :size="12" />演示一次更新<ArrowRight :size="11" />
        </button>
      </div>
      <p><LockKeyhole :size="11" />手动补充优先，AI 新发现需有依据。</p>
      <label class="check-label disabled-option"
        ><input type="checkbox" disabled />随新聊天自动更新<span
          >接入真实聊天后启用</span
        ></label
      >
    </footer>
    <ProfileEditor
      v-if="editorOpen"
      :item="editing"
      @close="editorOpen = false"
    />
  </aside>
</template>
