import type { Contact, ReplyInput } from "../types/workspace";

/** Group examples address selected topics only; custom text still requires the real model. */
export function groupReplies(contact: Contact, input: ReplyInput): string[] {
  const selected = contact.messages.filter(
    (message) =>
      input.selectedMessageIds?.includes(message.id) &&
      message.direction === "other",
  );
  if (!selected.length) {
    throw new Error("先选中至少一条要回复的群消息");
  }
  const member = contact.members?.find(
    (entry) => entry.id === input.replyTargetId,
  );
  if (input.replyTargetId && (!member || member.id === "self")) {
    throw new Error("请选择群内的回复对象");
  }
  const prefix = member ? `@${member.name} ` : "";
  const budget = selected.some((message) =>
    ["group-1", "group-4"].includes(message.id),
  );
  const schedule = selected.some((message) =>
    ["group-2", "group-5"].includes(message.id),
  );
  const topic =
    budget && schedule
      ? "核心范围和排期我一起核对一下"
      : budget
        ? "我先梳理必须保留的核心功能，再看其他部分怎么精简"
        : "我先确认范围清单的进度，再跟大家对齐时间";
  return [
    `${prefix}${topic}，确认后在群里同步。`,
    `${prefix}收到，${budget ? "核心功能优先保留，具体精简方式我再确认。" : "我先看下剩余工作量，确认后给一个准确时间。"}`,
    `${prefix}${input.tone === "俏皮" ? "先把事情捋顺，别让预算和排期打架。" : "这部分我来核对，"}${schedule ? "时间确认好了就同步，先不乱拍板。" : "我整理好范围后再发群里，大家一起看看。"}`,
  ];
}
