use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    /// Exposes a classified message without including raw network, database or secret values.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
    pub fn storage(_: rusqlite::Error) -> Self {
        Self::new("STORAGE_ERROR", "本地资料保存失败，请重试")
    }
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }
}
pub type AppResult<T> = Result<T, AppError>;

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_millis() as i64)
        .unwrap_or(0)
}
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub direction: String,
    pub text: String,
    pub at: String,
    #[serde(default)]
    pub sender_id: Option<String>,
    #[serde(default)]
    pub sender_name: Option<String>,
    #[serde(default)]
    pub reply_to_id: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    #[serde(default = "private_kind")]
    pub kind: String,
    #[serde(default)]
    pub members: Vec<GroupMember>,
    pub id: String,
    pub name: String,
    pub initial: String,
    pub relationship: String,
    pub tint: String,
    pub description: String,
    pub messages: Vec<ChatMessage>,
}
fn private_kind() -> String {
    "private".into()
}
#[derive(Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub message_id: String,
    pub quote: String,
    pub at: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileItem {
    pub id: String,
    pub category: String,
    pub subject_key: String,
    pub subject_label: String,
    pub content: String,
    pub source_kind: String,
    pub locked: bool,
    pub scope: String,
    pub expires_at_ms: Option<i64>,
    pub revision: i64,
    pub evidence: Vec<Evidence>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub id: String,
    pub kind: String,
    pub category: String,
    pub subject_key: String,
    pub subject_label: String,
    pub content: String,
    pub target_item_id: Option<String>,
    pub evidence: Vec<Evidence>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileEvent {
    pub id: String,
    pub item_id: Option<String>,
    pub summary: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub created_at_ms: i64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exclusion {
    pub id: String,
    pub subject_key: String,
    pub subject_label: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactProfile {
    pub items: Vec<ProfileItem>,
    pub proposals: Vec<Proposal>,
    pub events: Vec<ProfileEvent>,
    pub exclusions: Vec<Exclusion>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub contacts: Vec<Contact>,
    pub profiles: HashMap<String, ContactProfile>,
}

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ProfileAction {
    Save {
        contact_id: String,
        item: ProfileItem,
    },
    Delete {
        contact_id: String,
        item_id: String,
        exclude_future: bool,
    },
    Decide {
        contact_id: String,
        proposal_id: String,
        decision: String,
        expires_at_ms: Option<i64>,
    },
    RemoveExclusion {
        contact_id: String,
        exclusion_id: String,
    },
    DemoUpdate {
        contact_id: String,
    },
}
impl ProfileAction {
    pub fn contact_id(&self) -> &str {
        match self {
            Self::Save { contact_id, .. }
            | Self::Delete { contact_id, .. }
            | Self::Decide { contact_id, .. }
            | Self::RemoveExclusion { contact_id, .. }
            | Self::DemoUpdate { contact_id } => contact_id,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSummary {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub configured: bool,
    pub consent: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInput {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub consent: bool,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyInput {
    pub request_id: String,
    pub contact_id: String,
    pub task: String,
    pub tone: String,
    pub custom_tone: String,
    pub background: String,
    pub use_profile: bool,
    pub manual_transcript: Option<String>,
    #[serde(default)]
    pub selected_message_ids: Vec<String>,
    #[serde(default)]
    pub reply_target_id: Option<String>,
}
#[derive(Serialize)]
pub struct ReplyResult {
    pub candidates: Vec<String>,
    pub mode: &'static str,
}

/// Fixtures contain invented conversations only, and are never confused with live WeChat data.
pub fn demo_workspace() -> AppResult<Workspace> {
    serde_json::from_str(include_str!("../../../shared/demo-workspace.json"))
        .map_err(|_| AppError::new("FIXTURE_ERROR", "示例数据载入失败"))
}

/// Keeps recent information time-bound and rejects invalid categories before entering a transaction.
pub fn validate_item(item: &ProfileItem) -> AppResult<()> {
    if ![
        "relationship",
        "communication",
        "preference",
        "boundary",
        "recent",
    ]
    .contains(&item.category.as_str())
    {
        return Err(AppError::validation("不支持的资料分类"));
    }
    if item.content.trim().is_empty() || item.content.chars().count() > 1000 {
        return Err(AppError::validation("资料内容需要 1～1,000 字符"));
    }
    if item.subject_key.is_empty()
        || item.subject_key.len() > 150
        || item.subject_label.chars().count() > 100
    {
        return Err(AppError::validation("资料主题格式不正确"));
    }
    if !["long_term", "recent"].contains(&item.scope.as_str()) {
        return Err(AppError::validation("资料生效范围不正确"));
    }
    if item.scope == "recent" && item.expires_at_ms.is_none_or(|time| time <= now_ms()) {
        return Err(AppError::validation("请为近期事项选择未来的到期时间"));
    }
    if item.category == "recent" && item.scope != "recent" {
        return Err(AppError::validation("近期事项需要设置到期时间"));
    }
    Ok(())
}
