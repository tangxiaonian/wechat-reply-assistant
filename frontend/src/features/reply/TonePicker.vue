<script setup lang="ts">
import { ref, onMounted } from "vue";
import { SlidersHorizontal, Settings2 } from "@lucide/vue";
import { loadToneTemplates, errorMessage } from "../../bridge";
import { useWorkspace } from "../../stores/workspace";
const store = useWorkspace();
const custom = ref(false);
onMounted(async () => {
  try {
    store.toneTemplates = await loadToneTemplates();
  } catch (error) {
    store.error = errorMessage(error);
  }
});
</script>

<template>
  <section class="tone-picker">
    <div class="field-title">
      <label>用什么语气</label
      ><button
        class="text-button"
        :aria-expanded="custom"
        @click="custom = !custom"
      >
        <SlidersHorizontal :size="12" />本次补充
      </button>
    </div>
    <div class="tone-options" role="group" aria-label="回复语气">
      <button
        v-for="tone in store.availableTones"
        :key="tone.id"
        :aria-pressed="store.chosenTone?.id === tone.id"
        @click="
          store.selectedToneId = tone.id;
          store.tone = tone.baseTone ?? '自然';
        "
      >
        {{ tone.name }}
      </button>
    </div>
    <p v-if="!store.availableTones.length" class="helper">
      暂无已开启语气，请到设置里新增或开启。
    </p>
    <button
      class="text-button tone-settings-link"
      @click="
        store.settingsTab = 'tones';
        store.settingsOpen = true;
      "
    >
      <Settings2 :size="12" />管理语气 · 在设置中新增和开关
    </button>
    <input
      v-if="custom"
      v-model="store.customTone"
      class="custom-tone"
      maxlength="300"
      placeholder="例如：直接一点，不用表情，也别太客套"
      aria-label="自定义语气"
    />
  </section>
</template>
