use crate::{
    application::{profiles, replies},
    domain::*,
    infrastructure::{model, storage},
};
use rusqlite::Connection;

#[test]
fn native_state_opens_without_model_credentials_and_preserves_settings() {
    use crate::infrastructure::sync_settings::{self, SyncSettings};
    use std::os::unix::fs::PermissionsExt;

    // Exercise native startup, including the shared HTTP client, in an isolated data directory.
    let directory = std::env::temp_dir().join(format!("echo-startup-{}", new_id()));
    {
        let state = storage::AppState::open(&directory).unwrap();
        let db = state.connection().unwrap();
        assert!(!storage::provider(&db).unwrap().configured);
        assert!(state.requests.lock().unwrap().is_empty());
        sync_settings::apply(
            &db,
            Some(SyncSettings {
                enabled: true,
                interval_seconds: 15,
            }),
        )
        .unwrap();
        assert_eq!(
            std::fs::metadata(directory.join("assistant.sqlite3"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    {
        let state = storage::AppState::open(&directory).unwrap();
        let db = state.connection().unwrap();
        assert_eq!(
            sync_settings::apply(&db, None).unwrap().interval_seconds,
            15
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn sync_cadence_persists_and_rejects_invalid_intervals() {
    use crate::infrastructure::sync_settings::{self, SyncSettings};
    let db = database();
    assert!(!sync_settings::apply(&db, None).unwrap().enabled);
    assert!(sync_settings::apply(
        &db,
        Some(SyncSettings {
            enabled: true,
            interval_seconds: 1
        })
    )
    .is_err());
    sync_settings::apply(
        &db,
        Some(SyncSettings {
            enabled: true,
            interval_seconds: 30,
        }),
    )
    .unwrap();
    assert!(sync_settings::apply(&db, None).unwrap().enabled);
    assert_eq!(
        sync_settings::apply(&db, None).unwrap().interval_seconds,
        30
    );
}

#[test]
fn group_selection_keeps_member_identity_and_never_uses_profiles() {
    let mut workspace = demo_workspace().unwrap();
    let mut request = input();
    request.contact_id = "demo-group".into();
    request.use_profile = true;
    assert!(replies::messages(&request, &workspace).is_err());
    request.selected_message_ids = vec!["group-1".into()];
    request.reply_target_id = Some("lin".into());
    workspace.profiles.insert(
        "demo-group".into(),
        workspace.profiles["demo-friend"].clone(),
    );
    let payload = replies::messages(&request, &workspace).unwrap();
    let user: serde_json::Value =
        serde_json::from_str(payload[1]["content"].as_str().unwrap()).unwrap();
    assert!(user["profile"].as_array().unwrap().is_empty());
    assert_eq!(user["groupReply"]["recipient"]["name"], "林敏");
    assert_eq!(user["history"][0]["sender"], "林敏");
    request.selected_message_ids = vec!["friend-1".into()];
    assert!(replies::messages(&request, &workspace).is_err());
    request.selected_message_ids = vec!["group-1".into()];
    request.reply_target_id = Some("outsider".into());
    assert!(replies::messages(&request, &workspace).is_err());
    let db = database();
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM contacts WHERE id='demo-group'",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn tone_templates_keep_enable_state_in_settings_without_touching_profiles() {
    use crate::infrastructure::tones::{self, ToneAction, ToneTemplate};
    let db = database();
    let baseline = tones::apply(&db, None).unwrap();
    assert_eq!(baseline.len(), 4);
    let list = tones::apply(
        &db,
        Some(ToneAction::Save {
            template: ToneTemplate {
                id: new_id(),
                name: "懒人直球".into(),
                instruction: "一句话说重点".into(),
                enabled: true,
                base_tone: None,
            },
        }),
    )
    .unwrap();
    let mut custom = list
        .iter()
        .find(|entry| entry.name == "懒人直球")
        .unwrap()
        .clone();
    custom.enabled = false;
    tones::apply(
        &db,
        Some(ToneAction::Save {
            template: custom.clone(),
        }),
    )
    .unwrap();
    assert!(
        !tones::apply(&db, None)
            .unwrap()
            .iter()
            .find(|entry| entry.id == custom.id)
            .unwrap()
            .enabled
    );
    tones::apply(&db, Some(ToneAction::Delete { id: custom.id })).unwrap();
    assert_eq!(tones::apply(&db, None).unwrap().len(), 4);
    assert_eq!(
        storage::load(&db).unwrap().profiles["demo-friend"]
            .items
            .len(),
        4
    );
}

fn database() -> Connection {
    let mut db = Connection::open_in_memory().expect("temporary SQLite");
    storage::initialize(&mut db).expect("initial migration and demo seed");
    db
}

#[test]
fn migration_is_repeatable_and_does_not_reseed_deleted_data() {
    let mut db = database();
    profiles::apply(
        &mut db,
        ProfileAction::Delete {
            contact_id: "demo-friend".into(),
            item_id: "friend-style".into(),
            exclude_future: false,
        },
    )
    .unwrap();
    storage::initialize(&mut db).unwrap();
    let workspace = storage::load(&db).unwrap();
    assert!(!workspace.profiles["demo-friend"]
        .items
        .iter()
        .any(|item| item.id == "friend-style"));
    assert!(db
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none());
}

#[test]
fn conflict_can_become_recent_without_overwriting_the_locked_original() {
    let mut db = database();
    let result = profiles::apply(
        &mut db,
        ProfileAction::Decide {
            contact_id: "demo-friend".into(),
            proposal_id: "friend-conflict".into(),
            decision: "accept_as_recent".into(),
            expires_at_ms: Some(now_ms() + 86_400_000),
        },
    )
    .unwrap();
    let profile = &result.profiles["demo-friend"];
    assert_eq!(
        profile
            .items
            .iter()
            .find(|item| item.id == "friend-style")
            .unwrap()
            .content,
        "喜欢简短轻松的回复，可以偶尔用表情。"
    );
    assert!(profile
        .items
        .iter()
        .any(|item| item.scope == "recent" && item.content.contains("少用表情")));
    assert!(profile
        .events
        .iter()
        .any(|event| event.summary.contains("近期")));
    assert_eq!(result.profiles["demo-work"].items.len(), 3);
}

#[test]
fn recent_acceptance_requires_a_future_expiry_and_rolls_back() {
    let mut db = database();
    let result = profiles::apply(
        &mut db,
        ProfileAction::Decide {
            contact_id: "demo-friend".into(),
            proposal_id: "friend-conflict".into(),
            decision: "accept_as_recent".into(),
            expires_at_ms: None,
        },
    );
    assert!(result.is_err());
    assert_eq!(
        storage::load(&db).unwrap().profiles["demo-friend"]
            .proposals
            .len(),
        2
    );
}

#[test]
fn deleting_excluded_topic_cleans_evidence_and_history_and_blocks_reintroduction() {
    let mut db = database();
    let result = profiles::apply(
        &mut db,
        ProfileAction::DemoUpdate {
            contact_id: "demo-friend".into(),
        },
    )
    .unwrap();
    let item = result.profiles["demo-friend"]
        .items
        .iter()
        .find(|item| item.subject_key == "preference.demo_update")
        .unwrap();
    let id = item.id.clone();
    let result = profiles::apply(
        &mut db,
        ProfileAction::Delete {
            contact_id: "demo-friend".into(),
            item_id: id.clone(),
            exclude_future: true,
        },
    )
    .unwrap();
    assert!(!result.profiles["demo-friend"]
        .events
        .iter()
        .any(|event| event.item_id.as_deref() == Some(&id)));
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM profile_evidence WHERE item_id=?1",
            [&id],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert!(profiles::apply(
        &mut db,
        ProfileAction::DemoUpdate {
            contact_id: "demo-friend".into()
        }
    )
    .is_err());
}

#[test]
fn stale_manual_edit_is_rejected() {
    let mut db = database();
    let mut item = storage::load(&db).unwrap().profiles["demo-friend"].items[0].clone();
    item.content = "新的称呼".into();
    item.revision = 0;
    let error = profiles::apply(
        &mut db,
        ProfileAction::Save {
            contact_id: "demo-friend".into(),
            item,
        },
    )
    .err()
    .unwrap();
    assert_eq!(error.code, "PROFILE_VERSION_CONFLICT");
}

fn input() -> ReplyInput {
    ReplyInput {
        request_id: new_id(),
        contact_id: "demo-friend".into(),
        task: "reply".into(),
        tone: "自然".into(),
        custom_tone: String::new(),
        background: String::new(),
        use_profile: false,
        manual_transcript: None,
        selected_message_ids: vec![],
        reply_target_id: None,
    }
}

#[test]
fn manual_context_preserves_sender_and_blocks_reply_to_self() {
    let messages = replies::parse_manual("对方：周六见？\n我：可以\n我得晚半小时").unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].direction, "self");
    let mut input = input();
    input.manual_transcript = Some("对方：周六见？\n我：可以".into());
    assert_eq!(
        replies::messages(&input, &demo_workspace().unwrap())
            .unwrap_err()
            .code,
        "ACTION_REQUIRED"
    );
    input.task = "rewrite".into();
    assert!(replies::messages(&input, &demo_workspace().unwrap()).is_ok());
    assert!(replies::parse_manual("未标明发送人的消息").is_err());
}

#[test]
fn profile_disabled_means_no_profile_content_and_no_pending_observation() {
    let workspace = demo_workspace().unwrap();
    let payload = replies::messages(&input(), &workspace).unwrap();
    let user: serde_json::Value =
        serde_json::from_str(payload[1]["content"].as_str().unwrap()).unwrap();
    assert!(user["profile"].as_array().unwrap().is_empty());
    let mut with_profile = input();
    with_profile.use_profile = true;
    let payload = replies::messages(&with_profile, &workspace).unwrap();
    let text = payload[1]["content"].as_str().unwrap();
    assert!(text.contains("可以偶尔用表情"));
    assert!(!text.contains("近期回复少用表情"));
    assert!(!text.contains("主动给出具体时间和地点"));
}

#[test]
fn invalid_model_output_is_not_treated_as_success() {
    assert_eq!(
        replies::parse_candidates("```json\n{\"candidates\":[\"一\",\"二\",\"三\"]}\n```")
            .unwrap()
            .len(),
        3
    );
    assert!(replies::parse_candidates("{\"candidates\":[\"只有一条\"]}").is_err());
    assert!(replies::parse_candidates("这不是结构化候选").is_err());
}

#[test]
fn provider_requires_https_and_does_not_accept_keys_in_urls() {
    assert_eq!(
        model::endpoint("https://example.test/v1/").unwrap(),
        "https://example.test/v1/chat/completions"
    );
    assert!(model::endpoint("http://example.test/v1").is_err());
    assert!(model::endpoint("https://user:password@example.test/v1").is_err());
    assert!(model::endpoint("https://example.test/v1?key=secret").is_err());
}
