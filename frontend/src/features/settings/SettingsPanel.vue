<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import {
  ArrowUpRight,
  Check,
  Info,
  KeyRound,
  LoaderCircle,
  PlugZap,
  ShieldCheck,
  X,
} from "@lucide/vue";
import {
  native,
  saveProvider,
  removeProvider,
  testProvider,
  errorMessage,
} from "../../bridge";
import { useWorkspace } from "../../stores/workspace";
import ToneSettings from "./ToneSettings.vue";
import SyncSettings from "./SyncSettings.vue";
const store = useWorkspace();
const name = ref(store.provider.name);
const baseUrl = ref(store.provider.baseUrl);
const model = ref(store.provider.model);
const key = ref("");
const consent = ref(store.provider.consent);
const busy = ref(false);
const result = ref("");
const error = ref("");
const heading = ref<HTMLHeadingElement>();
watch([baseUrl, model], () => {
  consent.value = false;
  result.value = "";
});
onMounted(() => {
  heading.value?.focus();
});

/** The component keeps the newly entered key only until an explicit native save completes. */
async function save(): Promise<void> {
  busy.value = true;
  error.value = "";
  result.value = "";
  try {
    store.provider = await saveProvider({
      name: name.value,
      baseUrl: baseUrl.value,
      model: model.value,
      apiKey: key.value,
      consent: consent.value,
    });
    key.value = "";
    result.value = "已保存到系统钥匙串与本地配置";
    store.notify("模型配置已保存，可返回主面板选择使用模型");
  } catch (failure) {
    error.value = errorMessage(failure);
  } finally {
    busy.value = false;
  }
}
async function test(): Promise<void> {
  busy.value = true;
  error.value = "";
  result.value = "";
  try {
    result.value = `连接成功 · ${((await testProvider()) / 1000).toFixed(1)} 秒 · 测试不含聊天正文`;
  } catch (failure) {
    error.value = errorMessage(failure);
  } finally {
    busy.value = false;
  }
}
async function remove(): Promise<void> {
  busy.value = true;
  try {
    await removeProvider();
    store.provider = {
      name: "",
      baseUrl: "",
      model: "",
      configured: false,
      consent: false,
    };
    store.useModel = false;
    name.value = "";
    baseUrl.value = "";
    model.value = "";
    key.value = "";
    consent.value = false;
    result.value = "已移除模型配置和钥匙串项目";
  } catch (failure) {
    error.value = errorMessage(failure);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <section
    class="settings-panel"
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-title"
  >
    <header>
      <div>
        <span class="eyebrow">偏好设置</span>
        <h2 id="settings-title" ref="heading" tabindex="-1">
          {{
            store.settingsTab === "tones"
              ? "管理你的语气"
              : store.settingsTab === "sync"
                ? "自动同步"
                : "连接你的模型"
          }}
        </h2>
      </div>
      <button
        class="icon-button"
        aria-label="关闭设置"
        @click="store.settingsOpen = false"
      >
        <X :size="18" />
      </button>
    </header>
    <nav class="settings-tabs">
      <button
        :aria-pressed="store.settingsTab === 'connection'"
        @click="store.settingsTab = 'connection'"
      >
        模型连接</button
      ><button
        :aria-pressed="store.settingsTab === 'tones'"
        @click="store.settingsTab = 'tones'"
      >
        语气管理
      </button>
      <button
        :aria-pressed="store.settingsTab === 'sync'"
        @click="store.settingsTab = 'sync'"
      >
        同步设置
      </button>
    </nav>
    <ToneSettings
      v-if="store.settingsTab === 'tones'"
      class="settings-scroll"
    />
    <SyncSettings
      v-else-if="store.settingsTab === 'sync'"
      class="settings-scroll"
    />
    <div v-else class="settings-scroll">
      <div class="settings-intro">
        <PlugZap :size="19" />
        <p>
          让回复，真正结合你的想法。<span
            >支持 Chat Completions 风格的 HTTPS 服务。</span
          >
        </p>
      </div>
      <div v-if="!native" class="info-banner">
        <Info :size="15" /><span
          >浏览器仅预览界面。请在桌面版填写 Key，使用系统钥匙串保存。</span
        >
      </div>
      <form @submit.prevent="save">
        <label
          >服务商名称<input
            v-model="name"
            maxlength="100"
            placeholder="例如：我的模型服务"
            required
            :disabled="!native" /></label
        ><label
          >API 基础地址<input
            v-model="baseUrl"
            type="url"
            placeholder="https://你的服务地址/v1"
            autocomplete="off"
            required
            :disabled="!native" /></label
        ><label
          >模型名称<input
            v-model="model"
            maxlength="200"
            placeholder="填写服务商提供的 model 标识"
            required
            :disabled="!native" /></label
        ><label
          >API Key<span v-if="store.provider.configured" class="field-hint"
            >已保存；留空保持原 Key</span
          ><input
            v-model="key"
            type="password"
            placeholder="仅保存到 macOS 钥匙串"
            autocomplete="new-password"
            :required="!store.provider.configured"
            :disabled="!native"
        /></label>
        <div class="keychain-note">
          <KeyRound :size="13" />已有 Key 不会读回界面，也不进入日志。
        </div>
        <label class="consent-option"
          ><input v-model="consent" type="checkbox" :disabled="!native" /><span
            >允许向该服务发送本次聊天片段、本次背景，以及我启用的沟通档案，用于生成回复。<small
              >默认示例会话；手动导入的内容也会按此范围处理。</small
            ></span
          ></label
        >
        <p v-if="error" class="inline-error" role="alert">{{ error }}</p>
        <p v-if="result" class="connection-result" role="status">
          <Check :size="14" />{{ result }}
        </p>
        <div class="settings-buttons">
          <button
            class="button primary"
            :disabled="busy || !native"
            type="submit"
          >
            <LoaderCircle v-if="busy" class="spin" :size="14" />保存配置</button
          ><button
            class="button"
            :disabled="busy || !store.provider.configured || !native"
            type="button"
            @click="test"
          >
            测试已保存连接</button
          ><button
            v-if="store.provider.configured"
            class="text-button danger"
            type="button"
            :disabled="busy"
            @click="remove"
          >
            移除
          </button>
        </div>
      </form>
      <section class="wechat-connection">
        <div>
          <span class="connection-icon">微</span><strong>微信数据通道</strong
          ><span class="mini-label">本版待接入</span>
        </div>
        <p>
          数据库读取路线已在本机验证；这一版先体验页面和档案流程，不读取真实会话，也不会发送消息。
        </p>
        <span class="helper"
          >后续按稳定账号 ID 接入只读查询，不依赖 OCR 或旁白。</span
        >
      </section>
      <div class="settings-privacy">
        <ShieldCheck :size="16" />
        <p>
          本机资料由你管理<span
            >桌面档案存于本地 SQLite，MVP
            不做数据库级加密。微信密钥不进入本应用配置。</span
          >
        </p>
      </div>
    </div>
    <footer class="settings-footer">
      <span>回声 · v0.1.0 体验版</span
      ><button class="text-button" @click="store.settingsOpen = false">
        返回回复<ArrowUpRight :size="12" />
      </button>
    </footer>
  </section>
</template>
