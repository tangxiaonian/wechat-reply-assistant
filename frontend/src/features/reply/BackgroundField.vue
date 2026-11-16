<script setup lang="ts">
import { useWorkspace } from "../../stores/workspace";
const store = useWorkspace();
</script>
<template>
  <section class="background-control">
    <label class="field-title background-switch"
      ><span>本次补充背景<span class="optional">选填</span></span
      ><span
        ><input
          v-model="store.backgroundEnabled"
          type="checkbox"
          aria-label="开启本次补充背景"
        />{{ store.backgroundEnabled ? "已开启" : "已关闭" }}</span
      ></label
    >
    <textarea
      v-if="store.backgroundEnabled"
      id="reply-background"
      v-model="store.background"
      rows="2"
      maxlength="2000"
      aria-label="本次补充背景"
      placeholder="想表达什么，或有什么需要注意的？"
    ></textarea>
    <span v-if="store.backgroundEnabled" class="helper"
      >只用于这次回复；关闭开关后不传给模型。</span
    >
  </section>
</template>
