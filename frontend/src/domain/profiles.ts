import type {
  ContactProfile,
  ProfileAction,
  ProfileEvent,
  ProfileItem,
} from "../types/workspace";

/** Applies preview actions without overwriting a locked entry unless the user explicitly accepts. */
export function applyProfileAction(
  original: ContactProfile,
  action: ProfileAction,
): ContactProfile {
  const profile = structuredClone(original);
  let before: string | null = null;
  let after: string | null = null;
  let summary = "";
  let itemId: string | null = null;
  if (action.type === "save") {
    if (!action.item.content.trim()) {
      throw new Error("请填写资料内容");
    }
    if (action.item.content.length > 1000) {
      throw new Error("资料内容最多 1,000 字符");
    }
    if (
      action.item.scope === "recent" &&
      (!action.item.expiresAtMs || action.item.expiresAtMs <= Date.now())
    ) {
      throw new Error("请为近期事项选择未来的到期时间");
    }
    const existing = profile.items.find((item) => item.id === action.item.id);
    if (existing && existing.revision !== action.item.revision) {
      throw new Error("资料已更新，请重新打开编辑");
    }
    before = existing?.content ?? null;
    const unchanged = existing?.content === action.item.content.trim();
    const item: ProfileItem = {
      ...action.item,
      content: action.item.content.trim(),
      sourceKind: unchanged ? existing.sourceKind : "manual",
      revision: (existing?.revision ?? 0) + 1,
      evidence: unchanged ? existing.evidence : [],
    };
    profile.items = [
      ...profile.items.filter((entry) => entry.id !== item.id),
      item,
    ];
    summary = existing ? "你修改了资料" : "你补充了资料";
    itemId = item.id;
    after = item.content;
  } else if (action.type === "delete") {
    const item = profile.items.find((entry) => entry.id === action.itemId);
    if (!item) {
      return profile;
    }
    const removeIds = new Set(
      profile.items
        .filter((entry) =>
          action.excludeFuture
            ? entry.subjectKey === item.subjectKey
            : entry.id === item.id,
        )
        .map((entry) => entry.id),
    );
    profile.items = profile.items.filter((entry) => !removeIds.has(entry.id));
    profile.proposals = profile.proposals.filter((proposal) =>
      action.excludeFuture
        ? proposal.subjectKey !== item.subjectKey
        : proposal.targetItemId !== item.id,
    );
    profile.events = profile.events.filter(
      (event) => !event.itemId || !removeIds.has(event.itemId),
    );
    if (
      action.excludeFuture &&
      !profile.exclusions.some((entry) => entry.subjectKey === item.subjectKey)
    ) {
      profile.exclusions.push({
        id: crypto.randomUUID(),
        subjectKey: item.subjectKey,
        subjectLabel: item.subjectLabel,
      });
    }
    return profile;
  } else if (action.type === "remove_exclusion") {
    profile.exclusions = profile.exclusions.filter(
      (item) => item.id !== action.exclusionId,
    );
    return profile;
  } else if (action.type === "demo_update") {
    const key = "preference.demo_update";
    if (profile.exclusions.some((item) => item.subjectKey === key)) {
      throw new Error("这项示例信息已被你排除，不会再次加入");
    }
    if (profile.items.some((item) => item.subjectKey === key)) {
      throw new Error("这组示例没有更多新信息");
    }
    const content =
      action.contactId === "demo-friend"
        ? "看展前希望先吃点东西。"
        : action.contactId === "demo-work"
          ? "资料优先提供 PDF 版，方便审阅。"
          : "更希望先看精简方案，再决定下一步。";
    const item: ProfileItem = {
      id: crypto.randomUUID(),
      category: "preference",
      subjectKey: key,
      subjectLabel: "模拟新增偏好",
      content,
      sourceKind: "chat_fact",
      locked: false,
      scope: "long_term",
      expiresAtMs: null,
      revision: 1,
      evidence: [
        {
          messageId: "demo-new-message",
          quote: content,
          at: "模拟新消息，非真实聊天",
        },
      ],
    };
    profile.items.push(item);
    itemId = item.id;
    after = content;
    summary = "演示更新：加入一条带来源的偏好";
  } else {
    const proposal = profile.proposals.find(
      (entry) => entry.id === action.proposalId,
    );
    if (!proposal) {
      throw new Error("这项建议已处理，请刷新后查看");
    }
    const target = profile.items.find(
      (entry) => entry.id === proposal.targetItemId,
    );
    if (
      action.decision === "accept" ||
      action.decision === "accept_as_recent"
    ) {
      if (
        profile.exclusions.some(
          (entry) => entry.subjectKey === proposal.subjectKey,
        )
      ) {
        throw new Error("这项内容已排除，请先解除排除");
      }
      const recent = action.decision === "accept_as_recent";
      if (recent && (!action.expiresAtMs || action.expiresAtMs <= Date.now())) {
        throw new Error("请先选择近期偏好的到期日期");
      }
      before = recent ? null : (target?.content ?? null);
      const item: ProfileItem = {
        id: recent || !target ? crypto.randomUUID() : target.id,
        category: recent ? "recent" : proposal.category,
        subjectKey: proposal.subjectKey,
        subjectLabel: proposal.subjectLabel,
        content: proposal.content,
        sourceKind: "user_confirmed",
        locked: true,
        scope: recent ? "recent" : "long_term",
        expiresAtMs: recent ? (action.expiresAtMs ?? null) : null,
        revision: recent ? 1 : (target?.revision ?? 0) + 1,
        evidence: proposal.evidence,
      };
      profile.items = [
        ...profile.items.filter(
          (entry) =>
            entry.id !== item.id &&
            !(
              recent &&
              entry.subjectKey === item.subjectKey &&
              entry.scope === "recent"
            ),
        ),
        item,
      ];
      itemId = item.id;
      after = item.content;
      summary = recent
        ? "你将新信息加入近期偏好，保留原资料"
        : target
          ? "你采纳了新信息并替换原资料"
          : "你确认了 AI 观察";
    } else {
      summary =
        action.decision === "keep_existing"
          ? "你保留了原资料，未采纳这次变化"
          : "你忽略了一条观察";
    }
    profile.proposals = profile.proposals.filter(
      (entry) => entry.id !== proposal.id,
    );
  }
  const event: ProfileEvent = {
    id: crypto.randomUUID(),
    itemId,
    summary,
    before,
    after,
    createdAtMs: Date.now(),
  };
  profile.events.unshift(event);
  return profile;
}

/** Returns only current, confirmed profile entries; pending proposals are never included. */
export function activeProfileItems(
  profile: ContactProfile,
  now = Date.now(),
): ProfileItem[] {
  return profile.items.filter(
    (item) => item.expiresAtMs === null || item.expiresAtMs > now,
  );
}
