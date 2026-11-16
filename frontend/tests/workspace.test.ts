import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import fixture from "../../shared/demo-workspace.json";
import { useWorkspace } from "../src/stores/workspace";
import * as bridge from "../src/bridge";
import type { ReplyResult } from "../src/types/workspace";

vi.mock("../src/bridge", () => ({
  loadWorkspace: async () => structuredClone(fixture),
  getProvider: async () => ({
    name: "测试模型",
    baseUrl: "https://example.test/v1",
    model: "fixture",
    configured: true,
    consent: true,
  }),
  generateReply: vi.fn(),
  cancelReply: vi.fn(async () => {}),
  windowAction: vi.fn(async () => {}),
  errorMessage: (error: unknown) =>
    error instanceof Error ? error.message : "错误",
}));

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
});

describe("reply session isolation", () => {
  it("omits disabled background and preserves drafts during automatic context refresh", async () => {
    const store = useWorkspace();
    await store.initialize();
    store.useModel = true;
    store.background = "不应该被发送的背景";
    store.backgroundEnabled = false;
    vi.mocked(bridge.generateReply).mockResolvedValue({
      candidates: ["一", "二", "三"],
      mode: "model",
    });
    await store.generate();
    expect(vi.mocked(bridge.generateReply).mock.lastCall?.[0].background).toBe(
      "",
    );
    store.backgroundEnabled = true;
    await store.generate();
    expect(vi.mocked(bridge.generateReply).mock.lastCall?.[0].background).toBe(
      "不应该被发送的背景",
    );
    store.draft = "正在编辑的草稿";
    await store.refreshContext(true);
    expect(store.draft).toBe("正在编辑的草稿");
    expect(store.candidates).toHaveLength(3);
  });
  it("separates group selection and never sends a person profile in group mode", async () => {
    const store = useWorkspace();
    await store.initialize();
    await store.selectContact("demo-group");
    expect(store.isGroup).toBe(true);
    expect(store.profileOpen).toBe(false);
    expect(store.profile).toBeUndefined();
    await store.generate();
    expect(store.error).toContain("先选中");
    await store.changeGroupReply(["group-1"], "lin");
    store.reference = true;
    store.useModel = true;
    vi.mocked(bridge.generateReply).mockResolvedValue({
      candidates: ["一", "二", "三"],
      mode: "model",
    });
    await store.generate();
    expect(vi.mocked(bridge.generateReply).mock.lastCall?.[0]).toMatchObject({
      useProfile: false,
      selectedMessageIds: ["group-1"],
      replyTargetId: "lin",
    });
    await store.selectContact("demo-friend");
    expect(store.profileOpen).toBe(true);
    expect(store.selectedMessageIds).toEqual([]);
  });
  it("uses only enabled styles and includes the selected custom instruction", async () => {
    const store = useWorkspace();
    await store.initialize();
    store.toneTemplates = [
      {
        id: "casual",
        name: "别太正经",
        instruction: "短句，善意吐槽，不写官话",
        enabled: true,
      },
    ];
    store.selectedToneId = "casual";
    store.useModel = true;
    vi.mocked(bridge.generateReply).mockResolvedValue({
      candidates: ["一", "二", "三"],
      mode: "model",
    });
    await store.generate();
    expect(vi.mocked(bridge.generateReply).mock.lastCall?.[0]).toMatchObject({
      tone: "别太正经",
      customTone: "短句，善意吐槽，不写官话",
    });
    store.toneTemplates[0]!.enabled = false;
    expect(store.availableTones).toEqual([]);
    await store.generate();
    expect(store.error).toContain("开启至少一种语气");
  });
  it("keeps a demo result labelled as demo even when the next request switches to a model", async () => {
    const store = useWorkspace();
    await store.initialize();
    await store.generate();
    expect(store.generatedMode).toBe("demo");
    store.useModel = true;
    expect(store.generatedMode).toBe("demo");
  });

  it("drops a late model response after the user selects a different contact", async () => {
    let finish!: (value: ReplyResult) => void;
    vi.mocked(bridge.generateReply).mockReturnValue(
      new Promise<ReplyResult>((resolve) => {
        finish = resolve;
      }),
    );
    const store = useWorkspace();
    await store.initialize();
    store.useModel = true;
    store.background = "只用于旧联系人";
    store.draft = "旧草稿";
    const pending = store.generate();
    await store.selectContact("demo-work");
    finish({ candidates: ["旧候选1", "旧候选2", "旧候选3"], mode: "model" });
    await pending;
    expect(bridge.cancelReply).toHaveBeenCalledOnce();
    expect(store.selectedId).toBe("demo-work");
    expect(store.background).toBe("");
    expect(store.draft).toBe("");
    expect(store.candidates).toEqual([]);
    expect(store.generating).toBe(false);
  });
});
