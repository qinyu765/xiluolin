use std::{collections::BTreeMap, time::Duration};

use serde::Serialize;

use super::error::{ProviderError, ProviderErrorKind};

pub fn post_json<T: Serialize>(
    provider: &str,
    model: &str,
    base_url: &str,
    endpoint: &str,
    api_key: &str,
    headers: &BTreeMap<&str, &str>,
    body: &T,
    timeout: Duration,
) -> Result<serde_json::Value, ProviderError> {
    validate_cloud_settings(provider, model, base_url, api_key)?;
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|error| request_error(provider, model, error))?;
    let mut request = client
        .post(append_endpoint(base_url, endpoint))
        .bearer_auth(api_key.trim())
        .json(body);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = request
        .send()
        .map_err(|error| request_error(provider, model, error))?;
    let status = response.status();
    if !status.is_success() {
        return Err(http_error(provider, model, status.as_u16()));
    }
    response.json::<serde_json::Value>().map_err(|_| {
        ProviderError::new(
            provider,
            model,
            ProviderErrorKind::InvalidResponse,
            Some(status.as_u16()),
            "响应不是有效 JSON",
        )
    })
}

pub fn validate_cloud_settings(
    provider: &str,
    model: &str,
    base_url: &str,
    api_key: &str,
) -> Result<(), ProviderError> {
    if api_key.trim().is_empty() {
        return Err(ProviderError::new(
            provider,
            model,
            ProviderErrorKind::Configuration,
            None,
            "API Key 不能为空",
        ));
    }
    if base_url.trim().is_empty() {
        return Err(ProviderError::new(
            provider,
            model,
            ProviderErrorKind::Configuration,
            None,
            "Base URL 不能为空",
        ));
    }
    if model.trim().is_empty() {
        return Err(ProviderError::new(
            provider,
            model,
            ProviderErrorKind::Configuration,
            None,
            "模型不能为空",
        ));
    }
    Ok(())
}

pub fn append_endpoint(base_url: &str, endpoint: &str) -> String {
    let base_url = base_url.trim_end_matches('/');
    let endpoint = endpoint.trim_start_matches('/');
    if base_url.ends_with(endpoint) {
        base_url.to_string()
    } else {
        format!("{base_url}/{endpoint}")
    }
}

pub fn http_error(provider: &str, model: &str, status: u16) -> ProviderError {
    let kind = match status {
        401 | 403 => ProviderErrorKind::Authentication,
        429 => ProviderErrorKind::RateLimited,
        _ => ProviderErrorKind::RemoteFailure,
    };
    ProviderError::new(
        provider,
        model,
        kind,
        Some(status),
        format!("远端服务返回 HTTP {status}"),
    )
}

pub fn request_error(provider: &str, model: &str, error: reqwest::Error) -> ProviderError {
    let kind = if error.is_timeout() {
        ProviderErrorKind::Timeout
    } else {
        ProviderErrorKind::Network
    };
    ProviderError::new(
        provider,
        model,
        kind,
        error.status().map(|status| status.as_u16()),
        if error.is_timeout() {
            "Provider 请求超时"
        } else {
            "Provider 网络请求失败"
        },
    )
}
