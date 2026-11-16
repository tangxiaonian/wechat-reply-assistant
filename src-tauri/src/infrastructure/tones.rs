use crate::domain::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToneTemplate {
    pub id: String,
    pub name: String,
    pub instruction: String,
    pub enabled: bool,
    pub base_tone: Option<String>,
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToneAction {
    Save { template: ToneTemplate },
    Delete { id: String },
}

/// Style definitions live in application settings, never in a person or group profile.
pub fn apply(db: &Connection, action: Option<ToneAction>) -> AppResult<Vec<ToneTemplate>> {
    let saved: Option<String> = db
        .query_row(
            "SELECT value_json FROM app_settings WHERE key='tone_templates'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::storage)?;
    let mut list: Vec<ToneTemplate> = serde_json::from_str(
        saved
            .as_deref()
            .unwrap_or(include_str!("../../../shared/tone-defaults.json")),
    )
    .map_err(|_| AppError::validation("语气配置格式无效"))?;
    if let Some(action) = action {
        match action {
            ToneAction::Save { mut template } => {
                template.name = template.name.trim().into();
                template.instruction = template.instruction.trim().into();
                if template.name.is_empty()
                    || template.name.chars().count() > 20
                    || template.instruction.is_empty()
                    || template.instruction.chars().count() > 300
                {
                    return Err(AppError::validation(
                        "语气名称限 1～20 字，描述限 1～300 字",
                    ));
                }
                if let Some(index) = list.iter().position(|entry| entry.id == template.id) {
                    template.base_tone = list[index].base_tone.clone();
                    list[index] = template;
                } else {
                    template.id = new_id();
                    template.base_tone = None;
                    list.push(template);
                }
            }
            ToneAction::Delete { id } => {
                list.retain(|entry| entry.id != id || entry.base_tone.is_some());
            }
        }
        let json =
            serde_json::to_string(&list).map_err(|_| AppError::validation("语气配置无法保存"))?;
        db.execute("INSERT INTO app_settings(key,value_json,updated_at_ms) VALUES('tone_templates',?1,?2) ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json,updated_at_ms=excluded.updated_at_ms",params![json,now_ms()]).map_err(AppError::storage)?;
    }
    Ok(list)
}
