use crate::domain::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::{collections::HashMap, path::Path, sync::Mutex};
use tokio::sync::oneshot;

pub struct AppState {
    pub http_client: reqwest::Client,
    pub db: Mutex<Connection>,
    pub requests: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

impl AppState {
    /// Opens only the app-owned database; it has no connection to WeChat's source files.
    pub fn open(directory: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        use std::os::unix::fs::PermissionsExt;
        std::fs::create_dir_all(directory)?;
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))?;
        let path = directory.join("assistant.sqlite3");
        let mut db = Connection::open(&path)?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        initialize(&mut db).map_err(|_| "Application database initialization failed")?;
        Ok(Self {
            // One client owns the reusable connection pool; each request still has a 45s timeout.
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(45))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            db: Mutex::new(db),
            requests: Mutex::new(HashMap::new()),
        })
    }

    pub fn connection(&self) -> AppResult<std::sync::MutexGuard<'_, Connection>> {
        self.db
            .lock()
            .map_err(|_| AppError::new("STORAGE_ERROR", "本地资料暂时不可用"))
    }
}

/// Applies the initial immutable migration once, then seeds clearly identified demo contacts.
pub fn initialize(db: &mut Connection) -> AppResult<()> {
    db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=3000;")
        .map_err(AppError::storage)?;
    let version: i64 = db
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(AppError::storage)?;
    if version > 1 {
        return Err(AppError::new(
            "STORAGE_ERROR",
            "数据版本较新，请使用匹配版本的应用",
        ));
    }
    if version == 0 {
        let tx = db.transaction().map_err(AppError::storage)?;
        tx.execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .map_err(AppError::storage)?;
        tx.execute("INSERT INTO schema_migrations(version,name,checksum,applied_at_ms) VALUES(1,'initial-preview','v0.1',?1)", [now_ms()]).map_err(AppError::storage)?;
        let workspace = demo_workspace()?;
        tx.execute("INSERT INTO local_accounts(id,source_kind,source_account_key,display_name,query_profile_name,source_root,source_version,created_at_ms,updated_at_ms) VALUES('demo','demo','demo','示例账号','demo','demo://local','0.1',?1,?1)", [now_ms()]).map_err(AppError::storage)?;
        for contact in workspace
            .contacts
            .iter()
            .filter(|entry| entry.kind != "group")
        {
            tx.execute("INSERT INTO contacts(id,account_id,source_peer_id,display_name,created_at_ms,updated_at_ms) VALUES(?1,'demo',?1,?2,?3,?3)", params![contact.id, contact.name, now_ms()]).map_err(AppError::storage)?;
            tx.execute("INSERT INTO conversations(id,account_id,contact_id,source_conversation_id,title,created_at_ms,updated_at_ms) VALUES(?1,'demo',?1,?1,?2,?3,?3)", params![contact.id, contact.name, now_ms()]).map_err(AppError::storage)?;
            tx.execute(
                "INSERT INTO contact_profile_settings(contact_id,updated_at_ms) VALUES(?1,?2)",
                params![contact.id, now_ms()],
            )
            .map_err(AppError::storage)?;
            if let Some(profile) = workspace.profiles.get(&contact.id) {
                for item in &profile.items {
                    insert_item(&tx, &contact.id, item)?;
                }
                for proposal in &profile.proposals {
                    insert_pending(&tx, &contact.id, proposal)?;
                }
            }
        }
        tx.pragma_update(None, "user_version", 1)
            .map_err(AppError::storage)?;
        tx.commit().map_err(AppError::storage)?;
    }
    Ok(())
}

/// Inserts a confirmed item and minimal evidence; raw chat windows are never stored.
pub fn insert_item(db: &Connection, contact_id: &str, item: &ProfileItem) -> AppResult<()> {
    db.execute("INSERT INTO profile_items(id,contact_id,category,subject_key,subject_label,content,source_kind,locked,scope,valid_from_ms,expires_at_ms,revision,created_at_ms,updated_at_ms) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?10,?10)", params![item.id, contact_id, item.category, item.subject_key, item.subject_label, item.content, item.source_kind, item.locked, item.scope, now_ms(), item.expires_at_ms, item.revision]).map_err(AppError::storage)?;
    if !item.evidence.is_empty() {
        let proposal_id = new_id();
        db.execute("INSERT INTO profile_proposals(id,contact_id,conversation_id,kind,operation,applied_item_id,category,subject_key,subject_label,proposed_content,scope,status,decision,dedupe_key,extractor_version,created_at_ms,resolved_at_ms) VALUES(?1,?2,?2,'fact','add',?3,?4,?5,?6,?7,?8,'applied','auto_apply',?1,'preview-v1',?9,?9)", params![proposal_id, contact_id, item.id, item.category, item.subject_key, item.subject_label, item.content, item.scope, now_ms()]).map_err(AppError::storage)?;
        insert_evidence(db, &proposal_id, Some(&item.id), contact_id, &item.evidence)?;
    }
    Ok(())
}

pub fn insert_pending(db: &Connection, contact_id: &str, proposal: &Proposal) -> AppResult<()> {
    db.execute("INSERT INTO profile_proposals(id,contact_id,conversation_id,kind,operation,target_item_id,target_revision,category,subject_key,subject_label,proposed_content,dedupe_key,extractor_version,created_at_ms) VALUES(?1,?2,?2,?3,?4,?5,?6,?7,?8,?9,?10,?1,'preview-v1',?11)", params![proposal.id, contact_id, proposal.kind, if proposal.target_item_id.is_some() { "update" } else { "add" }, proposal.target_item_id, proposal.target_item_id.as_ref().map(|_| 1), proposal.category, proposal.subject_key, proposal.subject_label, proposal.content, now_ms()]).map_err(AppError::storage)?;
    insert_evidence(db, &proposal.id, None, contact_id, &proposal.evidence)
}

pub fn insert_evidence(
    db: &Connection,
    proposal_id: &str,
    item_id: Option<&str>,
    conversation_id: &str,
    evidence: &[Evidence],
) -> AppResult<()> {
    for entry in evidence.iter().take(3) {
        let quote: String = entry.quote.chars().take(300).collect();
        db.execute("INSERT INTO profile_evidence(id,proposal_id,item_id,conversation_id,source_message_id,sender_direction,quote,created_at_ms) VALUES(?1,?2,?3,?4,?5,'other',?6,?7)", params![new_id(), proposal_id, item_id, conversation_id, entry.message_id, quote, now_ms()]).map_err(AppError::storage)?;
    }
    Ok(())
}

fn evidence(db: &Connection, field: &str, id: &str) -> AppResult<Vec<Evidence>> {
    let sql = match field {
        "item_id" => "SELECT source_message_id,quote FROM profile_evidence WHERE item_id=?1",
        _ => "SELECT source_message_id,quote FROM profile_evidence WHERE proposal_id=?1",
    };
    let mut statement = db.prepare(sql).map_err(AppError::storage)?;
    let result = statement
        .query_map([id], |row| {
            Ok(Evidence {
                message_id: row.get(0)?,
                quote: row.get(1)?,
                at: "示例消息来源".into(),
            })
        })
        .map_err(AppError::storage)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::storage);
    result
}

/// Rebuilds public DTOs from normalized records, keeping fixture conversations in memory only.
pub fn load(db: &Connection) -> AppResult<Workspace> {
    let mut workspace = demo_workspace()?;
    for contact in workspace
        .contacts
        .iter()
        .filter(|entry| entry.kind != "group")
    {
        let mut profile = ContactProfile::default();
        let mut statement = db.prepare("SELECT id,category,subject_key,subject_label,content,source_kind,locked,scope,expires_at_ms,revision FROM profile_items WHERE contact_id=?1 AND status='active' ORDER BY created_at_ms,id").map_err(AppError::storage)?;
        profile.items = statement
            .query_map([&contact.id], |row| {
                Ok(ProfileItem {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    subject_key: row.get(2)?,
                    subject_label: row.get(3)?,
                    content: row.get(4)?,
                    source_kind: row.get(5)?,
                    locked: row.get(6)?,
                    scope: row.get(7)?,
                    expires_at_ms: row.get(8)?,
                    revision: row.get(9)?,
                    evidence: vec![],
                })
            })
            .map_err(AppError::storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::storage)?;
        for item in &mut profile.items {
            item.evidence = evidence(db, "item_id", &item.id)?;
        }
        let mut statement = db.prepare("SELECT id,kind,category,subject_key,subject_label,proposed_content,target_item_id FROM profile_proposals WHERE contact_id=?1 AND status='pending' ORDER BY created_at_ms,id").map_err(AppError::storage)?;
        profile.proposals = statement
            .query_map([&contact.id], |row| {
                Ok(Proposal {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    category: row.get(2)?,
                    subject_key: row.get(3)?,
                    subject_label: row.get(4)?,
                    content: row.get(5)?,
                    target_item_id: row.get(6)?,
                    evidence: vec![],
                })
            })
            .map_err(AppError::storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::storage)?;
        for proposal in &mut profile.proposals {
            proposal.evidence = evidence(db, "proposal_id", &proposal.id)?;
        }
        let mut statement = db.prepare("SELECT id,item_id,action,before_json,after_json,created_at_ms FROM profile_events WHERE contact_id=?1 ORDER BY created_at_ms DESC,id DESC LIMIT 200").map_err(AppError::storage)?;
        profile.events = statement
            .query_map([&contact.id], |row| {
                let before: Option<String> = row.get(3)?;
                let after: Option<String> = row.get(4)?;
                let decode = |value: Option<String>| {
                    value
                        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                        .and_then(|v| v.get("content").and_then(|v| v.as_str()).map(String::from))
                };
                let action: String = row.get(2)?;
                let summary = match action.as_str() {
                    "manual_add" => "你补充了资料",
                    "manual_edit" => "你修改了资料",
                    "auto_add" => "演示更新：加入一条偏好",
                    "accept" => "你采纳了新信息",
                    "accept_as_recent" => "你加入了近期偏好，保留原资料",
                    "keep_existing" => "你保留了原资料",
                    "ignore" => "你忽略了一条观察",
                    _ => "资料已更新",
                };
                Ok(ProfileEvent {
                    id: row.get(0)?,
                    item_id: row.get(1)?,
                    summary: summary.into(),
                    before: decode(before),
                    after: decode(after),
                    created_at_ms: row.get(5)?,
                })
            })
            .map_err(AppError::storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::storage)?;
        let mut statement = db.prepare("SELECT id,subject_key,subject_label FROM profile_exclusions WHERE contact_id=?1 ORDER BY created_at_ms DESC").map_err(AppError::storage)?;
        profile.exclusions = statement
            .query_map([&contact.id], |row| {
                Ok(Exclusion {
                    id: row.get(0)?,
                    subject_key: row.get(1)?,
                    subject_label: row.get(2)?,
                })
            })
            .map_err(AppError::storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::storage)?;
        workspace.profiles.insert(contact.id.clone(), profile);
    }
    Ok(workspace)
}

pub fn provider(db: &Connection) -> AppResult<ProviderSummary> {
    db.query_row("SELECT name,base_url,model,reply_consent_revision=config_revision FROM provider_configs WHERE id='primary'", [], |row| Ok(ProviderSummary { name: row.get(0)?, base_url: row.get(1)?, model: row.get(2)?, configured: true, consent: row.get::<_,Option<bool>>(3)?.unwrap_or(false) })).optional().map_err(AppError::storage).map(|value| value.unwrap_or_default())
}
