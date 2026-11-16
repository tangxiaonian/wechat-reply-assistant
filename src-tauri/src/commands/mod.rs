use crate::{
    application::{profiles, replies},
    domain::*,
    infrastructure::{
        model,
        storage::{self, AppState},
    },
};
use std::time::Instant;
use tauri::{LogicalSize, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Reads or updates the application's automatic context refresh cadence.
#[tauri::command]
pub fn sync_settings(
    state: State<AppState>,
    input: Option<crate::infrastructure::sync_settings::SyncSettings>,
) -> AppResult<crate::infrastructure::sync_settings::SyncSettings> {
    crate::infrastructure::sync_settings::apply(&*state.connection()?, input)
}

/// Reads or changes only the application's named tone settings.
#[tauri::command]
pub fn tone_templates(
    state: State<AppState>,
    input: Option<crate::infrastructure::tones::ToneAction>,
) -> AppResult<Vec<crate::infrastructure::tones::ToneTemplate>> {
    crate::infrastructure::tones::apply(&*state.connection()?, input)
}

#[tauri::command]
pub fn load_workspace(state: State<AppState>) -> AppResult<Workspace> {
    storage::load(&*state.connection()?)
}
#[tauri::command]
pub fn profile_action(state: State<AppState>, input: ProfileAction) -> AppResult<Workspace> {
    profiles::apply(&mut *state.connection()?, input)
}
#[tauri::command]
pub fn get_provider(state: State<AppState>) -> AppResult<ProviderSummary> {
    storage::provider(&*state.connection()?)
}
#[tauri::command]
pub fn save_provider(state: State<AppState>, input: ProviderInput) -> AppResult<ProviderSummary> {
    model::save(&*state.connection()?, input)
}
#[tauri::command]
pub fn remove_provider(state: State<AppState>) -> AppResult<()> {
    model::remove(&*state.connection()?)
}

#[tauri::command]
pub async fn test_provider(state: State<'_, AppState>) -> AppResult<u64> {
    let provider = { storage::provider(&*state.connection()?)? };
    let started = Instant::now();
    model::request(&state.http_client, &provider, vec![serde_json::json!({"role":"user","content":"请只回复 OK。这是连接测试，不含用户聊天。"})]).await?;
    Ok(started.elapsed().as_millis() as u64)
}

/// Runs one cancellable provider request; selection, consent and profile assembly stay native.
#[tauri::command]
pub async fn generate_reply(
    state: State<'_, AppState>,
    input: ReplyInput,
) -> AppResult<ReplyResult> {
    uuid::Uuid::parse_str(&input.request_id).map_err(|_| AppError::validation("请求标识无效"))?;
    let (provider, messages) = {
        let db = state.connection()?;
        let provider = storage::provider(&db)?;
        if !provider.consent {
            return Err(AppError::new(
                "MODEL_CONSENT_REQUIRED",
                "请在设置中确认模型的处理范围",
            ));
        }
        let messages = replies::messages(&input, &storage::load(&db)?)?;
        (provider, messages)
    };
    let (sender, receiver) = tokio::sync::oneshot::channel();
    {
        let mut requests = state
            .requests
            .lock()
            .map_err(|_| AppError::new("REQUEST_ERROR", "请求状态不可用"))?;
        if requests.contains_key(&input.request_id) {
            return Err(AppError::validation("请求标识已使用"));
        }
        requests.insert(input.request_id.clone(), sender);
    }
    let result = tokio::select! {
        _ = receiver => Err(AppError::new("CANCELLED", "已取消本次生成")),
        result = model::request(&state.http_client, &provider, messages) => result.and_then(|content| replies::parse_candidates(&content)).map(|candidates| ReplyResult{ candidates, mode:"model" }),
    };
    if let Ok(mut requests) = state.requests.lock() {
        requests.remove(&input.request_id);
    }
    result
}

#[tauri::command]
pub fn cancel_reply(state: State<AppState>, request_id: String) -> AppResult<()> {
    if let Some(sender) = state
        .requests
        .lock()
        .map_err(|_| AppError::new("REQUEST_ERROR", "请求状态不可用"))?
        .remove(&request_id)
    {
        let _ = sender.send(());
    }
    Ok(())
}

#[tauri::command]
pub fn copy_reply(app: tauri::AppHandle, text: String) -> AppResult<()> {
    if text.chars().count() > 20_000 {
        return Err(AppError::validation("复制内容过长"));
    }
    app.clipboard()
        .write_text(text)
        .map_err(|_| AppError::new("CLIPBOARD_ERROR", "复制失败，请重试"))
}

/// Exposes named window operations rather than generic system execution.
#[tauri::command]
pub fn window_action(app: tauri::AppHandle, action: String) -> AppResult<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::new("WINDOW_ERROR", "窗口不可用"))?;
    let result = match action.as_str() {
        "hide" => window.hide(),
        "minimize" => window.minimize(),
        "pin" => window.set_always_on_top(true),
        "unpin" => window.set_always_on_top(false),
        "expand" => window.set_size(LogicalSize::new(
            820.0,
            window
                .inner_size()
                .map(|size| size.height as f64 / window.scale_factor().unwrap_or(1.0))
                .unwrap_or(850.0),
        )),
        "collapse" => window.set_size(LogicalSize::new(
            420.0,
            window
                .inner_size()
                .map(|size| size.height as f64 / window.scale_factor().unwrap_or(1.0))
                .unwrap_or(850.0),
        )),
        _ => return Err(AppError::validation("不支持的窗口操作")),
    };
    result.map_err(|_| AppError::new("WINDOW_ERROR", "窗口操作失败，请重试"))
}
