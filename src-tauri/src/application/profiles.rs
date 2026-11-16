use crate::{domain::*, infrastructure::storage};
use rusqlite::{params, Connection};
use serde_json::json;

/// Changes one contact in a short transaction, preserving manual priority and deletion boundaries.
pub fn apply(db: &mut Connection, action: ProfileAction) -> AppResult<Workspace> {
    let contact_id = action.contact_id().to_owned();
    let workspace = storage::load(db)?;
    let profile = workspace
        .profiles
        .get(&contact_id)
        .ok_or_else(|| AppError::validation("没有找到联系人"))?;
    let tx = db.transaction().map_err(AppError::storage)?;
    match action {
        ProfileAction::Save { mut item, .. } => {
            validate_item(&item)?;
            let existing = profile.items.iter().find(|entry| entry.id == item.id);
            if let Some(old) = existing {
                if old.revision != item.revision {
                    return Err(AppError::new(
                        "PROFILE_VERSION_CONFLICT",
                        "资料已更新，请重新打开编辑",
                    ));
                }
                item.revision += 1;
                item.source_kind = if item.content == old.content {
                    old.source_kind.clone()
                } else {
                    "manual".into()
                };
                tx.execute("UPDATE profile_items SET category=?1,subject_label=?2,content=?3,source_kind=?4,locked=?5,scope=?6,expires_at_ms=?7,revision=revision+1,updated_at_ms=?8 WHERE id=?9 AND contact_id=?10 AND revision=?11", params![item.category, item.subject_label, item.content.trim(), item.source_kind, item.locked, item.scope, item.expires_at_ms, now_ms(), item.id, contact_id, old.revision]).map_err(AppError::storage)?;
                if item.content != old.content {
                    tx.execute("DELETE FROM profile_evidence WHERE item_id=?1", [&item.id])
                        .map_err(AppError::storage)?;
                }
                record(
                    &tx,
                    &contact_id,
                    Some(&item.id),
                    None,
                    "manual_edit",
                    Some(&old.content),
                    Some(&item.content),
                )?;
            } else {
                if profile
                    .items
                    .iter()
                    .any(|entry| entry.subject_key == item.subject_key && entry.scope == item.scope)
                {
                    return Err(AppError::validation("该主题已有资料，请编辑原条目"));
                }
                item.id = new_id();
                item.source_kind = "manual".into();
                item.revision = 1;
                item.evidence.clear();
                storage::insert_item(&tx, &contact_id, &item)?;
                record(
                    &tx,
                    &contact_id,
                    Some(&item.id),
                    None,
                    "manual_add",
                    None,
                    Some(&item.content),
                )?;
            }
        }
        ProfileAction::Delete {
            item_id,
            exclude_future,
            ..
        } => {
            let item = profile
                .items
                .iter()
                .find(|entry| entry.id == item_id)
                .ok_or_else(|| AppError::validation("资料不存在"))?;
            if exclude_future {
                tx.execute("INSERT OR IGNORE INTO profile_exclusions(id,contact_id,subject_key,subject_label,created_at_ms) VALUES(?1,?2,?3,?4,?5)", params![new_id(), contact_id, item.subject_key, item.subject_label, now_ms()]).map_err(AppError::storage)?;
                tx.execute(
                    "DELETE FROM profile_proposals WHERE contact_id=?1 AND subject_key=?2",
                    params![contact_id, item.subject_key],
                )
                .map_err(AppError::storage)?;
                tx.execute(
                    "DELETE FROM profile_items WHERE contact_id=?1 AND subject_key=?2",
                    params![contact_id, item.subject_key],
                )
                .map_err(AppError::storage)?;
            } else {
                tx.execute("DELETE FROM profile_proposals WHERE contact_id=?1 AND target_item_id=?2 AND status='pending'", params![contact_id, item_id]).map_err(AppError::storage)?;
                tx.execute(
                    "DELETE FROM profile_items WHERE id=?1 AND contact_id=?2",
                    params![item_id, contact_id],
                )
                .map_err(AppError::storage)?;
            }
        }
        ProfileAction::RemoveExclusion { exclusion_id, .. } => {
            tx.execute(
                "DELETE FROM profile_exclusions WHERE id=?1 AND contact_id=?2",
                params![exclusion_id, contact_id],
            )
            .map_err(AppError::storage)?;
        }
        ProfileAction::DemoUpdate { .. } => {
            let key = "preference.demo_update";
            if profile
                .exclusions
                .iter()
                .any(|entry| entry.subject_key == key)
            {
                return Err(AppError::validation("这项示例信息已被排除，不会再次加入"));
            }
            if profile.items.iter().any(|entry| entry.subject_key == key) {
                return Err(AppError::validation("这组示例没有更多新信息"));
            }
            let content = match contact_id.as_str() {
                "demo-friend" => "看展前希望先吃点东西。",
                "demo-work" => "资料优先提供 PDF 版，方便审阅。",
                _ => "更希望先看精简方案，再决定下一步。",
            };
            let item = ProfileItem {
                id: new_id(),
                category: "preference".into(),
                subject_key: key.into(),
                subject_label: "模拟新增偏好".into(),
                content: content.into(),
                source_kind: "chat_fact".into(),
                locked: false,
                scope: "long_term".into(),
                expires_at_ms: None,
                revision: 1,
                evidence: vec![Evidence {
                    message_id: "demo-new-message".into(),
                    quote: content.into(),
                    at: "模拟新消息".into(),
                }],
            };
            storage::insert_item(&tx, &contact_id, &item)?;
            record(
                &tx,
                &contact_id,
                Some(&item.id),
                None,
                "auto_add",
                None,
                Some(content),
            )?;
        }
        ProfileAction::Decide {
            proposal_id,
            decision,
            expires_at_ms,
            ..
        } => {
            let proposal = profile
                .proposals
                .iter()
                .find(|entry| entry.id == proposal_id)
                .ok_or_else(|| AppError::validation("这条建议已经处理"))?;
            if !["accept", "accept_as_recent", "keep_existing", "ignore"]
                .contains(&decision.as_str())
            {
                return Err(AppError::validation("不支持的核对操作"));
            }
            let target = profile
                .items
                .iter()
                .find(|entry| Some(&entry.id) == proposal.target_item_id.as_ref());
            let accepting = decision == "accept" || decision == "accept_as_recent";
            let mut applied_id: Option<String> = None;
            let mut before = None;
            if accepting {
                if profile
                    .exclusions
                    .iter()
                    .any(|entry| entry.subject_key == proposal.subject_key)
                {
                    return Err(AppError::validation("请先解除该事项的排除"));
                }
                let recent = decision == "accept_as_recent";
                let item = ProfileItem {
                    id: if recent {
                        new_id()
                    } else {
                        target.map(|entry| entry.id.clone()).unwrap_or_else(new_id)
                    },
                    category: if recent {
                        "recent".into()
                    } else {
                        proposal.category.clone()
                    },
                    subject_key: proposal.subject_key.clone(),
                    subject_label: proposal.subject_label.clone(),
                    content: proposal.content.clone(),
                    source_kind: "user_confirmed".into(),
                    locked: true,
                    scope: if recent {
                        "recent".into()
                    } else {
                        "long_term".into()
                    },
                    expires_at_ms: if recent { expires_at_ms } else { None },
                    revision: target.map_or(1, |entry| entry.revision + 1),
                    evidence: vec![],
                };
                validate_item(&item)?;
                if !recent && target.is_some() {
                    before = target.map(|entry| entry.content.as_str());
                    tx.execute("UPDATE profile_items SET content=?1,source_kind='user_confirmed',locked=1,revision=revision+1,updated_at_ms=?2 WHERE id=?3 AND contact_id=?4", params![item.content, now_ms(), item.id, contact_id]).map_err(AppError::storage)?;
                    tx.execute("DELETE FROM profile_evidence WHERE item_id=?1", [&item.id])
                        .map_err(AppError::storage)?;
                } else {
                    if recent {
                        tx.execute("UPDATE profile_items SET status='archived',revision=revision+1 WHERE contact_id=?1 AND subject_key=?2 AND scope='recent' AND status='active'", params![contact_id, item.subject_key]).map_err(AppError::storage)?;
                    }
                    storage::insert_item(&tx, &contact_id, &item)?;
                }
                tx.execute(
                    "UPDATE profile_evidence SET item_id=?1 WHERE proposal_id=?2",
                    params![item.id, proposal_id],
                )
                .map_err(AppError::storage)?;
                applied_id = Some(item.id);
            }
            tx.execute("UPDATE profile_proposals SET status=?1,decision=?2,applied_item_id=?3,resolved_at_ms=?4 WHERE id=?5 AND contact_id=?6", params![if accepting { "applied" } else { "rejected" }, decision, applied_id, now_ms(), proposal_id, contact_id]).map_err(AppError::storage)?;
            record(
                &tx,
                &contact_id,
                applied_id.as_deref(),
                Some(&proposal_id),
                &decision,
                before,
                if accepting {
                    Some(&proposal.content)
                } else {
                    None
                },
            )?;
        }
    }
    tx.execute("UPDATE contact_profile_settings SET profile_revision=profile_revision+1,updated_at_ms=?1 WHERE contact_id=?2", params![now_ms(), contact_id]).map_err(AppError::storage)?;
    tx.commit().map_err(AppError::storage)?;
    storage::load(db)
}

fn record(
    db: &Connection,
    contact: &str,
    item: Option<&str>,
    proposal: Option<&str>,
    action: &str,
    before: Option<&str>,
    after: Option<&str>,
) -> AppResult<()> {
    db.execute("INSERT INTO profile_events(id,contact_id,item_id,proposal_id,action,actor,before_json,after_json,created_at_ms) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![new_id(), contact, item, proposal, action, if action == "auto_add" { "system" } else { "user" }, before.map(|text| json!({"content":text}).to_string()), after.map(|text| json!({"content":text}).to_string()), now_ms()]).map_err(AppError::storage)?;
    Ok(())
}
