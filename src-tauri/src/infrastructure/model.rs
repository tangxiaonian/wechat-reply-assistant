use crate::{domain::*, infrastructure::storage};
use rusqlite::{params, Connection};
use security_framework::passwords::{
    delete_generic_password, generic_password, set_generic_password, PasswordOptions,
};
use serde_json::{json, Value};

const SERVICE: &str = "local.echo.wechat-reply.model";
const ACCOUNT: &str = "primary";

/// Rejects credentials in URLs and redirects; only an explicit HTTPS provider is supported.
pub fn endpoint(base: &str) -> AppResult<String> {
    let url = reqwest::Url::parse(base.trim())
        .map_err(|_| AppError::validation("请填写有效的 HTTPS 模型地址"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(AppError::validation(
            "模型地址须为 HTTPS，且不能包含凭据、查询参数或片段",
        ));
    }
    let base = base.trim().trim_end_matches('/');
    Ok(if base.ends_with("/chat/completions") {
        base.to_owned()
    } else {
        format!("{base}/chat/completions")
    })
}

/// Stores a newly entered key in Keychain and keeps only a reference in the application database.
pub fn save(db: &Connection, input: ProviderInput) -> AppResult<ProviderSummary> {
    endpoint(&input.base_url)?;
    if input.name.trim().is_empty() || input.model.trim().is_empty() || input.model.len() > 200 {
        return Err(AppError::validation("请填写服务商名称和模型名称"));
    }
    let previous = storage::provider(db)?;
    if input.api_key.trim().is_empty() && !previous.configured {
        return Err(AppError::validation("首次连接需要填写 API Key"));
    }
    if !input.api_key.trim().is_empty() {
        set_generic_password(SERVICE, ACCOUNT, input.api_key.trim().as_bytes()).map_err(|_| {
            AppError::new(
                "CREDENTIAL_UNAVAILABLE",
                "无法保存到系统钥匙串，请检查访问权限",
            )
        })?;
    }
    let consent = if input.consent { Some(1) } else { None };
    db.execute("INSERT INTO provider_configs(id,name,base_url,model,keychain_ref,is_default,config_revision,reply_consent_revision,created_at_ms,updated_at_ms) VALUES('primary',?1,?2,?3,?4,1,1,?5,?6,?6) ON CONFLICT(id) DO UPDATE SET name=excluded.name,base_url=excluded.base_url,model=excluded.model,config_revision=provider_configs.config_revision+1,reply_consent_revision=CASE WHEN excluded.reply_consent_revision IS NOT NULL THEN provider_configs.config_revision+1 ELSE NULL END,updated_at_ms=excluded.updated_at_ms", params![input.name.trim(), input.base_url.trim(), input.model.trim(), SERVICE, consent, now_ms()]).map_err(AppError::storage)?;
    storage::provider(db)
}

pub fn remove(db: &Connection) -> AppResult<()> {
    if let Err(error) = delete_generic_password(SERVICE, ACCOUNT) {
        if error.code() != -25300 {
            return Err(AppError::new(
                "CREDENTIAL_UNAVAILABLE",
                "无法从系统钥匙串移除，请重试",
            ));
        }
    }
    db.execute("DELETE FROM provider_configs WHERE id='primary'", [])
        .map_err(AppError::storage)?;
    Ok(())
}

/// Uses only the selected provider; neither prompts nor raw responses are written to logs.
pub async fn request(
    client: &reqwest::Client,
    provider: &ProviderSummary,
    messages: Vec<Value>,
) -> AppResult<String> {
    if !provider.configured {
        return Err(AppError::new("MODEL_NOT_CONFIGURED", "请先连接模型"));
    }
    let url = endpoint(&provider.base_url)?;
    let key = generic_password(PasswordOptions::new_generic_password(SERVICE, ACCOUNT)).map_err(
        |_| {
            AppError::new(
                "CREDENTIAL_UNAVAILABLE",
                "系统钥匙串中的模型 Key 不可用，请重新保存",
            )
        },
    )?;
    let key = String::from_utf8(key)
        .map_err(|_| AppError::new("CREDENTIAL_UNAVAILABLE", "模型 Key 格式无效"))?;
    let response = client
        .post(url)
        .bearer_auth(key)
        .json(&json!({"model":provider.model,"messages":messages}))
        .send()
        .await
        .map_err(|error| {
            if error.is_timeout() {
                AppError::new("MODEL_TIMEOUT", "模型请求已超时，可以重试")
            } else {
                AppError::new(
                    "MODEL_CONNECTION_FAILED",
                    "无法连接模型服务，请检查地址和网络",
                )
            }
        })?;
    if !response.status().is_success() {
        return Err(AppError::new(
            "MODEL_HTTP_ERROR",
            format!(
                "模型服务返回 HTTP {}，请检查配置或服务状态",
                response.status().as_u16()
            ),
        ));
    }
    let response: Value = response
        .json()
        .await
        .map_err(|_| AppError::new("MODEL_INVALID_RESPONSE", "模型响应不是支持的 JSON 格式"))?;
    response
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| AppError::new("MODEL_INVALID_RESPONSE", "当前服务的响应格式不兼容"))
}
