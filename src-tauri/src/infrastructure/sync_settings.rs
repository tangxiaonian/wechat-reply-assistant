use crate::domain::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSettings {
    pub enabled: bool,
    pub interval_seconds: u64,
}
/// Stores a bounded cadence without creating a background WeChat reader or model job.
pub fn apply(db: &Connection, input: Option<SyncSettings>) -> AppResult<SyncSettings> {
    if let Some(input) = input {
        if !(5..=300).contains(&input.interval_seconds) {
            return Err(AppError::validation("同步间隔需要是 5～300 秒的整数"));
        }
        let json =
            serde_json::to_string(&input).map_err(|_| AppError::validation("同步设置无法保存"))?;
        db.execute("INSERT INTO app_settings(key,value_json,updated_at_ms) VALUES('sync_settings',?1,?2) ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json,updated_at_ms=excluded.updated_at_ms",params![json,now_ms()]).map_err(AppError::storage)?;
        return Ok(input);
    }
    let saved: Option<String> = db
        .query_row(
            "SELECT value_json FROM app_settings WHERE key='sync_settings'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::storage)?;
    saved
        .map(|json| {
            serde_json::from_str(&json).map_err(|_| AppError::validation("同步设置格式无效"))
        })
        .unwrap_or(Ok(SyncSettings {
            enabled: false,
            interval_seconds: 30,
        }))
}
