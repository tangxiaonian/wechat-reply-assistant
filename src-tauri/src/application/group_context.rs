use crate::domain::*;
use serde_json::{json, Value};

/// Confirms selection and recipient against this group, independent of the latest overall sender.
pub fn validate(contact: &Contact, input: &ReplyInput) -> AppResult<()> {
    if input.manual_transcript.is_some() {
        return Err(AppError::validation(
            "本版群聊仅支持选择示例消息，暂不支持手动导入群片段",
        ));
    }
    if input.task == "rewrite" {
        return Err(AppError::validation("群聊请选择回复消息或在群里补充"));
    }
    if input.selected_message_ids.is_empty()
        || input.selected_message_ids.iter().any(|id| {
            !contact
                .messages
                .iter()
                .any(|message| &message.id == id && message.direction == "other")
        })
    {
        return Err(AppError::validation("请选择本群中其他成员发出的消息"));
    }
    if let Some(target) = &input.reply_target_id {
        if !contact
            .members
            .iter()
            .any(|member| &member.id == target && member.id != "self")
        {
            return Err(AppError::validation("回复对象不属于当前群聊"));
        }
    }
    Ok(())
}

/// Sends explicit targets as conversation data, with no persistent group or member profile.
pub fn payload(contact: &Contact, input: &ReplyInput) -> Value {
    json!({"selectedMessageIds":input.selected_message_ids,"recipient":input.reply_target_id.as_ref().and_then(|id|contact.members.iter().find(|member| &member.id==id)).map(|member|json!({"id":member.id,"name":member.name})).unwrap_or(json!({"scope":"whole_group"})),"instruction":"只回应选中的消息，其他消息用于消歧，不把其他成员的话当作同一人说的。不使用画像；不擅自承诺时间或费用。"})
}
