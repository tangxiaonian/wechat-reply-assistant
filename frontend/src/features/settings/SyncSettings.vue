<script setup lang="ts">
import { ref } from "vue";
import { useWorkspace } from "../../stores/workspace";
import { syncSettings, errorMessage } from "../../bridge";
const store = useWorkspace();
const enabled = ref(store.syncPreferences.enabled);
const seconds = ref(store.syncPreferences.intervalSeconds);
const error = ref("");
const busy = ref(false);
/** Saves only cadence preferences; reading still goes through the explicitly labelled data source. */
async function save(): Promise<void> {
  busy.value = true;
  error.value = "";
  try {
    store.syncPreferences = await syncSettings({
      enabled: enabled.value,
      intervalSeconds: seconds.value,
    });
    store.notify("同步设置已保存");
  } catch (failure) {
    error.value = errorMessage(failure);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="sync-settings">
    <p class="tone-intro">
      按你的节奏更新上下文。<span
        >保留手动刷新；自动同步默认关闭，保存后生效。</span
      >
    </p>
    <form @submit.prevent="save">
      <label class="check-label"
        ><input v-model="enabled" type="checkbox" />开启自动同步</label
      ><label
        >每多少秒同步一次<input
          v-model.number="seconds"
          type="number"
          min="5"
          max="300"
          step="1"
          required
      /></label>
      <p class="helper">
        可设置 5～300 秒，默认 30
        秒。窗口隐藏、生成中或使用手动片段时暂停；不会清空草稿，也不会调用模型分析。
      </p>
      <p class="info-banner">
        当前为示例数据源，自动同步仅重新载入示例，不代表获取到了微信新消息。真实读取通道仍待接入。
      </p>
      <p v-if="error" class="error-text" role="alert">{{ error }}</p>
      <button class="button primary" :disabled="busy" type="submit">
        保存同步设置
      </button>
    </form>
  </section>
</template>
