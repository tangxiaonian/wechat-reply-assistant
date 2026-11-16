import { describe, expect, it } from "vitest";
import fixture from "../../shared/demo-workspace.json";
import { activeProfileItems, applyProfileAction } from "../src/domain/profiles";
import { demoReplies } from "../src/domain/replies";
import type {
  ProfileItem,
  ReplyInput,
  Workspace,
} from "../src/types/workspace";

function workspace(): Workspace {
  return structuredClone(fixture) as Workspace;
}

describe("communication profile rules", () => {
  it("keeps the locked long-term item when a conflicting preference is accepted as recent", () => {
    const original = workspace().profiles["demo-friend"]!;
    const updated = applyProfileAction(original, {
      type: "decide",
      contactId: "demo-friend",
      proposalId: "friend-conflict",
      decision: "accept_as_recent",
      expiresAtMs: Date.now() + 86_400_000,
    });
    expect(
      updated.items.find((item) => item.id === "friend-style")?.content,
    ).toBe("喜欢简短轻松的回复，可以偶尔用表情。");
    expect(
      updated.items.find((item) => item.scope === "recent")?.content,
    ).toContain("少用表情");
    expect(
      updated.proposals.some((proposal) => proposal.id === "friend-conflict"),
    ).toBe(false);
    expect(original.proposals).toHaveLength(2);
  });
  it("requires a future expiry for a recent preference", () => {
    expect(() =>
      applyProfileAction(workspace().profiles["demo-friend"]!, {
        type: "decide",
        contactId: "demo-friend",
        proposalId: "friend-conflict",
        decision: "accept_as_recent",
      }),
    ).toThrow("到期日期");
  });
  it("does not include unconfirmed observations or expired entries in generation material", () => {
    const profile = workspace().profiles["demo-friend"]!;
    const recent: ProfileItem = {
      ...profile.items[0]!,
      id: "expired",
      scope: "recent",
      expiresAtMs: 1,
    };
    profile.items.push(recent);
    expect(
      activeProfileItems(profile).some((item) => item.id === "expired"),
    ).toBe(false);
    expect(
      activeProfileItems(profile).some((item) =>
        item.content.includes("主动给出具体"),
      ),
    ).toBe(false);
  });
  it("deletes associated content history and blocks a future demo update for the excluded topic", () => {
    let profile = applyProfileAction(workspace().profiles["demo-friend"]!, {
      type: "demo_update",
      contactId: "demo-friend",
    });
    const added = profile.items.find(
      (item) => item.subjectKey === "preference.demo_update",
    )!;
    expect(profile.events.some((event) => event.after === added.content)).toBe(
      true,
    );
    profile = applyProfileAction(profile, {
      type: "delete",
      contactId: "demo-friend",
      itemId: added.id,
      excludeFuture: true,
    });
    expect(profile.events.some((event) => event.after === added.content)).toBe(
      false,
    );
    expect(() =>
      applyProfileAction(profile, {
        type: "demo_update",
        contactId: "demo-friend",
      }),
    ).toThrow("排除");
  });
  it("rejects stale manual edits and does not change another contact", () => {
    const data = workspace();
    const profile = data.profiles["demo-friend"]!;
    const originalWork = JSON.stringify(data.profiles["demo-work"]);
    const item = { ...profile.items[0]!, content: "叫林同学就好", revision: 0 };
    expect(() =>
      applyProfileAction(profile, {
        type: "save",
        contactId: "demo-friend",
        item,
      }),
    ).toThrow("已更新");
    expect(JSON.stringify(data.profiles["demo-work"])).toBe(originalWork);
  });
});

describe("demo mode boundaries", () => {
  const input: ReplyInput = {
    requestId: "test",
    contactId: "demo-friend",
    task: "reply",
    tone: "自然",
    customTone: "",
    background: "我想晚半小时",
    useProfile: true,
    manualTranscript: null,
  };
  it("provides three scene-specific examples and distinguishes task branches", () => {
    const contact = workspace().contacts[0]!;
    expect(demoReplies(contact, input)).toHaveLength(3);
    expect(demoReplies(contact, input)[0]).toContain("两点半");
    expect(demoReplies(contact, { ...input, task: "rewrite" })[0]).toContain(
      "我周六下午有空",
    );
    expect(demoReplies(contact, { ...input, task: "follow_up" })[0]).toContain(
      "确认",
    );
  });
  it("requires a real model for arbitrary manual context", () => {
    expect(() =>
      demoReplies(workspace().contacts[0]!, {
        ...input,
        manualTranscript: "对方：测试",
      }),
    ).toThrow("需要连接模型");
  });
});
