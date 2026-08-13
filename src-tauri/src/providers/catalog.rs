use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ProviderOptionValue {
    Text(String),
    Boolean(bool),
    StringList(Vec<String>),
}

impl ProviderOptionValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_string_list(&self) -> Option<&[String]> {
        match self {
            Self::StringList(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderSettings {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub options: BTreeMap<String, ProviderOptionValue>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderRoutingConfig {
    #[serde(default)]
    pub primary: String,
    #[serde(default)]
    pub fallbacks: Vec<String>,
    #[serde(default)]
    pub settings: BTreeMap<String, ProviderSettings>,
}

impl ProviderRoutingConfig {
    pub fn provider_ids(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.primary.as_str()).chain(self.fallbacks.iter().map(String::as_str))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.primary.trim().is_empty() {
            return Err("必须选择 primary Provider".to_string());
        }
        if self.fallbacks.len() > 2 {
            return Err("Provider 调用链最多包含 3 项".to_string());
        }
        let mut seen = HashSet::new();
        for provider in self.provider_ids() {
            if !seen.insert(provider) {
                return Err(format!("Provider 调用链不能包含重复项：{provider}"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    Asr,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ProviderFieldKind {
    ApiKey,
    Text,
    Select,
    MultiSelect,
    Switch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderFieldChoice {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderFieldDescriptor {
    pub key: String,
    pub label: String,
    pub kind: ProviderFieldKind,
    pub required: bool,
    pub secret: bool,
    pub placeholder: String,
    pub help: String,
    pub choices: Vec<ProviderFieldChoice>,
    pub max_items: Option<u16>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderCapabilities {
    pub native_hotwords: bool,
    pub max_hotwords: Option<u16>,
    pub supports_prompt: bool,
    #[specta(type = Option<specta_typescript::Number>)]
    pub max_duration_ms: Option<u64>,
    pub local_model_management: bool,
    pub max_language_hints: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderDescriptor {
    pub id: String,
    pub name: String,
    pub capability: ProviderCapability,
    pub protocol: String,
    pub default_base_url: String,
    pub default_model: String,
    pub fields: Vec<ProviderFieldDescriptor>,
    pub capabilities: ProviderCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ProviderCatalog {
    pub asr: Vec<ProviderDescriptor>,
    pub text: Vec<ProviderDescriptor>,
}

pub fn provider_catalog() -> ProviderCatalog {
    ProviderCatalog {
        asr: vec![
            cloud_asr(
                "zhipu",
                "智谱 ASR",
                "multipart",
                "https://open.bigmodel.cn/api/paas/v4",
                "glm-asr-2512",
                true,
                Some(100),
            ),
            cloud_asr(
                "openai",
                "OpenAI-compatible ASR",
                "multipart",
                "https://api.openai.com/v1",
                "whisper-1",
                false,
                None,
            ),
            ProviderDescriptor {
                id: "local".to_string(),
                name: "本地 Whisper".to_string(),
                capability: ProviderCapability::Asr,
                protocol: "local-whisper".to_string(),
                default_base_url: String::new(),
                default_model: crate::local_asr_model::LOCAL_ASR_MODEL_NAME.to_string(),
                fields: Vec::new(),
                capabilities: ProviderCapabilities {
                    supports_prompt: true,
                    local_model_management: true,
                    ..ProviderCapabilities::default()
                },
            },
            qwen_audio_descriptor(),
            qwen3_asr_descriptor(),
        ],
        text: vec![
            cloud_text(
                "zhipu",
                "智谱文本",
                "https://open.bigmodel.cn/api/paas/v4",
                "glm-4.7-flash",
            ),
            cloud_text(
                "openai",
                "OpenAI-compatible 文本",
                "https://api.openai.com/v1",
                "gpt-4o-mini",
            ),
            cloud_text(
                "qwen",
                "千问文本",
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
                "qwen3.7-flash",
            ),
        ],
    }
}

fn cloud_asr(
    id: &str,
    name: &str,
    protocol: &str,
    base_url: &str,
    model: &str,
    native_hotwords: bool,
    max_hotwords: Option<u16>,
) -> ProviderDescriptor {
    ProviderDescriptor {
        id: id.to_string(),
        name: name.to_string(),
        capability: ProviderCapability::Asr,
        protocol: protocol.to_string(),
        default_base_url: base_url.to_string(),
        default_model: model.to_string(),
        fields: common_cloud_fields(base_url, model),
        capabilities: ProviderCapabilities {
            native_hotwords,
            max_hotwords,
            supports_prompt: true,
            max_duration_ms: (id == "zhipu").then_some(30_000),
            ..ProviderCapabilities::default()
        },
    }
}

fn cloud_text(id: &str, name: &str, base_url: &str, model: &str) -> ProviderDescriptor {
    ProviderDescriptor {
        id: id.to_string(),
        name: name.to_string(),
        capability: ProviderCapability::Text,
        protocol: "openai-chat".to_string(),
        default_base_url: base_url.to_string(),
        default_model: model.to_string(),
        fields: common_cloud_fields(base_url, model),
        capabilities: ProviderCapabilities::default(),
    }
}

fn qwen_audio_descriptor() -> ProviderDescriptor {
    let mut descriptor = cloud_asr(
        "qwen-audio",
        "Qwen-Audio 3.0 ASR",
        "dashscope-multimodal",
        "https://dashscope.aliyuncs.com",
        "qwen-audio-3.0-asr-flash",
        true,
        Some(100),
    );
    descriptor.fields.push(ProviderFieldDescriptor {
        key: "language_hints".to_string(),
        label: "语言提示".to_string(),
        kind: ProviderFieldKind::MultiSelect,
        required: false,
        secret: false,
        placeholder: "zh, en".to_string(),
        help: "最多选择 4 种可能语言".to_string(),
        choices: language_choices(),
        max_items: Some(4),
    });
    descriptor.capabilities.max_language_hints = Some(4);
    descriptor
}

fn qwen3_asr_descriptor() -> ProviderDescriptor {
    let mut descriptor = cloud_asr(
        "qwen3-asr",
        "Qwen3-ASR",
        "openai-chat-audio",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
        "qwen3-asr-flash",
        false,
        None,
    );
    descriptor.fields.extend([
        ProviderFieldDescriptor {
            key: "language".to_string(),
            label: "识别语言".to_string(),
            kind: ProviderFieldKind::Select,
            required: false,
            secret: false,
            placeholder: "自动检测".to_string(),
            help: "留空时自动检测语言".to_string(),
            choices: language_choices(),
            max_items: None,
        },
        ProviderFieldDescriptor {
            key: "enable_itn".to_string(),
            label: "启用逆文本正则化".to_string(),
            kind: ProviderFieldKind::Switch,
            required: false,
            secret: false,
            placeholder: String::new(),
            help: "默认关闭；开启后会将口述数字等转换为书写形式".to_string(),
            choices: Vec::new(),
            max_items: None,
        },
    ]);
    descriptor
}

fn common_cloud_fields(base_url: &str, model: &str) -> Vec<ProviderFieldDescriptor> {
    vec![
        field(
            "api_key",
            "API Key",
            ProviderFieldKind::ApiKey,
            true,
            true,
            "",
            "只保存到系统凭据库",
        ),
        field(
            "base_url",
            "Base URL",
            ProviderFieldKind::Text,
            true,
            false,
            base_url,
            "可填写公共或 Workspace 专属地域地址",
        ),
        field(
            "model",
            "模型",
            ProviderFieldKind::Text,
            true,
            false,
            model,
            "填写服务支持的模型 ID",
        ),
    ]
}

fn field(
    key: &str,
    label: &str,
    kind: ProviderFieldKind,
    required: bool,
    secret: bool,
    placeholder: &str,
    help: &str,
) -> ProviderFieldDescriptor {
    ProviderFieldDescriptor {
        key: key.to_string(),
        label: label.to_string(),
        kind,
        required,
        secret,
        placeholder: placeholder.to_string(),
        help: help.to_string(),
        choices: Vec::new(),
        max_items: None,
    }
}

fn language_choices() -> Vec<ProviderFieldChoice> {
    [
        ("zh", "中文"),
        ("en", "英语"),
        ("ja", "日语"),
        ("ko", "韩语"),
        ("yue", "粤语"),
    ]
    .into_iter()
    .map(|(value, label)| ProviderFieldChoice {
        value: value.to_string(),
        label: label.to_string(),
    })
    .collect()
}
