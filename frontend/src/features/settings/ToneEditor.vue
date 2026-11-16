<script setup lang="ts">
import { ref } from "vue";
import { X } from "@lucide/vue";
import type { ToneTemplate } from "../../types/workspace";
const props = defineProps<{ template: ToneTemplate | null; busy: boolean }>();
const emit = defineEmits<{ close: []; save: [template: ToneTemplate] }>();
const name = ref(props.template?.name ?? "");
const instruction = ref(props.template?.instruction ?? "");
const enabled = ref(props.template?.enabled ?? true);
const examples = [
  {
    name: "嘴贫但不冒犯",
    instruction: "像熟人接话，带一点善意吐槽，短句，别硬凹梗，不攻击人。",
  },
  {
    name: "懒人直球",
    instruction: "一句话说清重点，不寒暄，不绕弯，像平时随手回消息。",
  },
  {
    name: "温柔拒绝",
    instruction: "自然说明自己的安排和边界，温柔但坚定，不编借口，不反复道歉。",
  },
];
/** Only the style definition is saved; transient reply background never enters this form. */
function save(): void {
  emit("save", {
    id: props.template?.id ?? crypto.randomUUID(),
    name: name.value.trim(),
    instruction: instruction.value.trim(),
    enabled: enabled.value,
    baseTone: props.template?.baseTone,
  });
}
</script>
<template>
  <div class="tone-editor">
    <div class="field-title">
      <strong>{{ template ? "编辑语气" : "新增语气" }}</strong><button
        class="icon-button"
        aria-label="关闭语气编辑"
        @click="emit('close')"
      >
        <X :size="15" />
      </button>
    </div>
    <form @submit.prevent="save">
      <label
        >语气名称<input
          v-model="name"
          required
          maxlength="20"
          placeholder="例如：嘴贫但不冒犯"
      /></label>
      <label
        >希望怎么说<textarea
          v-model="instruction"
          required
          maxlength="300"
          rows="3"
          placeholder="写具体一点：短句、能开玩笑、别像写邮件…"
        ></textarea>
      </label>
      <div v-if="!template" class="tone-examples">
        <button
          v-for="example in examples"
          :key="example.name"
          type="button"
          class="text-button"
          @click="
            name = example.name;
            instruction = example.instruction;
          "
        >
          {{ example.name }}
        </button>
      </div>
      <label class="check-label"
        ><input v-model="enabled" type="checkbox" />开启后显示在聊天面板</label
      >
      <div class="form-actions">
        <button type="button" class="button compact" @click="emit('close')">
          取消</button
        ><button class="button primary compact" :disabled="busy" type="submit">
          保存语气
        </button>
      </div>
    </form>
  </div>
</template>
