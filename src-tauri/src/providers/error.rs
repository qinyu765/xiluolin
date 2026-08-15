use std::fmt;

use serde::{Deserialize, Serialize};

const MAX_SAFE_MESSAGE_CHARS: usize = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ProviderErrorKind {
    Configuration,
    IncompatibleInput,
    Authentication,
    RateLimited,
    Timeout,
    Network,
    RemoteFailure,
    LocalRuntime,
    InvalidResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ProviderErrorScope {
    Provider,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderError {
    pub provider: String,
    pub model: String,
    pub kind: ProviderErrorKind,
    pub scope: ProviderErrorScope,
    pub http_status: Option<u16>,
    pub message: String,
}

impl ProviderError {
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        kind: ProviderErrorKind,
        http_status: Option<u16>,
        message: impl AsRef<str>,
    ) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            kind,
            scope: ProviderErrorScope::Provider,
            http_status,
            message: safe_message(message.as_ref()),
        }
    }

    pub fn global(mut self) -> Self {
        self.scope = ProviderErrorScope::Global;
        self
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} / {} ({:?}{}): {}",
            self.provider,
            self.model,
            self.kind,
            self.http_status
                .map(|status| format!(", HTTP {status}"))
                .unwrap_or_default(),
            self.message
        )
    }
}

impl std::error::Error for ProviderError {}

fn safe_message(message: &str) -> String {
    let one_line = message.replace(['\r', '\n'], " ");
    let mut result = one_line.split_whitespace().collect::<Vec<_>>().join(" ");
    if let Some(index) = result.to_ascii_lowercase().find("bearer ") {
        result.truncate(index);
        result.push_str("[已脱敏]");
    }
    if result.chars().count() > MAX_SAFE_MESSAGE_CHARS {
        result = result
            .chars()
            .take(MAX_SAFE_MESSAGE_CHARS)
            .collect::<String>();
        result.push('…');
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRouteError {
    pub capability: &'static str,
    pub errors: Vec<ProviderError>,
}

impl fmt::Display for ProviderRouteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.errors.is_empty() {
            return write!(formatter, "{} Provider 调用失败", self.capability);
        }
        let summary = self
            .errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("；");
        write!(
            formatter,
            "{} Provider 全部失败：{}",
            self.capability, summary
        )
    }
}

impl std::error::Error for ProviderRouteError {}

#[cfg(test)]
mod tests {
    use super::{ProviderError, ProviderErrorKind, MAX_SAFE_MESSAGE_CHARS};

    #[test]
    fn provider_error_redacts_bearer_tokens_and_truncates_messages() {
        let error = ProviderError::new(
            "qwen",
            "model",
            ProviderErrorKind::Network,
            None,
            format!(
                "request failed\nAuthorization: Bearer secret-key {}",
                "x".repeat(400)
            ),
        );

        assert!(!error.message.contains("secret-key"));
        assert!(!error.message.contains('\n'));
        assert!(error.message.contains("[已脱敏]"));

        let long = ProviderError::new(
            "qwen",
            "model",
            ProviderErrorKind::RemoteFailure,
            None,
            "x".repeat(400),
        );
        assert_eq!(long.message.chars().count(), MAX_SAFE_MESSAGE_CHARS + 1);
        assert!(long.message.ends_with('…'));
    }
}
