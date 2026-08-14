use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use xiluolin_lib::providers::{
    asr::{default_asr_registry, route_asr, AsrInput},
    catalog::{ProviderOptionValue, ProviderRoutingConfig, ProviderSettings},
    error::ProviderErrorKind,
    text::{default_text_registry, route_text, TextInput},
};

fn temp_audio_path(extension: &str, bytes: &[u8]) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let path = std::env::temp_dir()
        .join("xiluolin-qwen-tests")
        .join(format!("audio-{nanos}.{extension}"));
    fs::create_dir_all(path.parent().expect("parent")).expect("create test directory");
    fs::write(&path, bytes).expect("write test audio");
    path
}

fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .expect("set read timeout");
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                request.extend_from_slice(&buffer[..count]);
                if let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..header_end]);
                    let content_length = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                                .flatten()
                        })
                        .unwrap_or(0);
                    if request.len() >= header_end + 4 + content_length {
                        break;
                    }
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(error) => panic!("read request: {error}"),
        }
    }
    request
}

fn spawn_server(
    status: &'static str,
    response_body: &'static str,
) -> (String, thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let base_url = format!("http://{}", listener.local_addr().expect("local address"));
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let request = read_request(&mut stream);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        request
    });
    (base_url, handle)
}

fn route(provider: &str, settings: ProviderSettings) -> ProviderRoutingConfig {
    ProviderRoutingConfig {
        primary: provider.to_string(),
        fallbacks: Vec::new(),
        settings: BTreeMap::from([(provider.to_string(), settings)]),
    }
}

fn settings(base_url: String, model: &str) -> ProviderSettings {
    ProviderSettings {
        api_key: "test-key".to_string(),
        base_url,
        model: model.to_string(),
        options: BTreeMap::new(),
    }
}

fn body_json(request: &[u8]) -> serde_json::Value {
    let body = request
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .map(|index| &request[index + 4..])
        .expect("request body");
    serde_json::from_slice(body).expect("valid JSON body")
}

#[test]
fn qwen_audio_sends_base64_hotwords_and_four_language_hints() {
    let audio_path = temp_audio_path("wav", b"fixture audio");
    let (base_url, handle) = spawn_server("200 OK", r#"{"output":{"text":"千问音频识别"}}"#);
    let mut provider_settings = settings(base_url, "qwen-audio-3.0-asr-flash");
    provider_settings.options.insert(
        "language_hints".to_string(),
        ProviderOptionValue::StringList(vec![
            "zh".to_string(),
            "en".to_string(),
            "ja".to_string(),
            "ko".to_string(),
            "fr".to_string(),
        ]),
    );
    let mut hotwords = vec![" XiLuoLin ".to_string(), "XiLuoLin".to_string()];
    hotwords.extend((0..120).map(|index| format!("词{index}")));
    let input = AsrInput {
        audio_path: audio_path.clone(),
        hotwords,
        context_prompt: None,
        local_model_path: None,
    };

    let result = route_asr(
        &input,
        &route("qwen-audio", provider_settings),
        &default_asr_registry(),
    )
    .expect("qwen audio request");
    let request = handle.join().expect("server result");
    let request_text = String::from_utf8_lossy(&request).to_ascii_lowercase();
    let body = body_json(&request);
    fs::remove_file(audio_path).expect("remove fixture");

    assert_eq!(result.output.text, "千问音频识别");
    assert!(request_text
        .starts_with("post /api/v1/services/aigc/multimodal-generation/generation http/1.1"));
    assert!(request_text.contains("authorization: bearer test-key"));
    assert!(request_text.contains("x-dashscope-sse: disable"));
    assert_eq!(body["model"], "qwen-audio-3.0-asr-flash");
    assert_eq!(
        body["input"]["messages"][0]["content"][0]["input_audio"]["data"],
        "data:audio/wav;base64,Zml4dHVyZSBhdWRpbw=="
    );
    assert_eq!(body["parameters"]["format"], "wav");
    assert_eq!(body["parameters"]["vocabulary"]["XiLuoLin"], 5);
    assert_eq!(
        body["parameters"]["vocabulary"]
            .as_object()
            .expect("vocabulary object")
            .len(),
        100
    );
    assert_eq!(
        body["parameters"]["language_hints"],
        serde_json::json!(["zh", "en", "ja", "ko"])
    );
}

#[test]
fn qwen3_asr_sends_system_glossary_language_and_disabled_itn() {
    let audio_path = temp_audio_path("mp3", b"fixture audio");
    let (base_url, handle) = spawn_server(
        "200 OK",
        r#"{"choices":[{"message":{"role":"assistant","content":"Qwen3 识别"}}]}"#,
    );
    let mut provider_settings =
        settings(format!("{base_url}/compatible-mode/v1/"), "qwen3-asr-flash");
    provider_settings.options.insert(
        "language".to_string(),
        ProviderOptionValue::Text("zh".to_string()),
    );
    provider_settings.options.insert(
        "enable_itn".to_string(),
        ProviderOptionValue::Boolean(false),
    );
    let input = AsrInput {
        audio_path: audio_path.clone(),
        hotwords: vec!["XiLuoLin".to_string(), "通义千问".to_string()],
        context_prompt: Some("语音输入应用".to_string()),
        local_model_path: None,
    };

    let result = route_asr(
        &input,
        &route("qwen3-asr", provider_settings),
        &default_asr_registry(),
    )
    .expect("qwen3 asr request");
    let request = handle.join().expect("server result");
    let request_text = String::from_utf8_lossy(&request).to_ascii_lowercase();
    let body = body_json(&request);
    fs::remove_file(audio_path).expect("remove fixture");

    assert_eq!(result.output.text, "Qwen3 识别");
    assert!(request_text.starts_with("post /compatible-mode/v1/chat/completions http/1.1"));
    assert_eq!(body["messages"][0]["role"], "system");
    assert!(body["messages"][0]["content"]
        .as_str()
        .expect("system content")
        .contains("XiLuoLin、通义千问"));
    assert_eq!(
        body["messages"][1]["content"][0]["input_audio"]["data"],
        "data:audio/mpeg;base64,Zml4dHVyZSBhdWRpbw=="
    );
    assert_eq!(body["stream"], false);
    assert_eq!(body["asr_options"]["language"], "zh");
    assert_eq!(body["asr_options"]["enable_itn"], false);
}

#[test]
fn qwen_text_disables_thinking_and_classifies_rate_limit() {
    let (base_url, handle) = spawn_server("429 Too Many Requests", r#"{"message":"slow down"}"#);
    let input = TextInput {
        raw_text: "需要整理的文字".to_string(),
        persona_id: "general".to_string(),
        persona_description: "自然清晰".to_string(),
        hotword_context: String::new(),
    };

    let result = route_text(
        &input,
        &route(
            "qwen",
            settings(format!("{base_url}/compatible-mode/v1"), "qwen3.7-flash"),
        ),
        &default_text_registry(),
    )
    .expect("text route falls back to raw text");
    let request = handle.join().expect("server result");
    let body = body_json(&request);

    assert_eq!(body["enable_thinking"], false);
    assert_eq!(body["model"], "qwen3.7-flash");
    assert_eq!(result.output.text, "需要整理的文字");
    assert_eq!(
        result.attempts[0].error_kind,
        Some(ProviderErrorKind::RateLimited)
    );
    assert_eq!(result.attempts[0].http_status, Some(429));
}

#[test]
fn qwen_text_classifies_invalid_json_and_empty_text() {
    let input = TextInput {
        raw_text: "保留原文".to_string(),
        persona_id: "general".to_string(),
        persona_description: "自然清晰".to_string(),
        hotword_context: String::new(),
    };

    for response in [
        "not-json",
        r#"{"choices":[{"message":{"role":"assistant","content":""}}]}"#,
    ] {
        let (base_url, handle) = spawn_server("200 OK", response);
        let result = route_text(
            &input,
            &route(
                "qwen",
                settings(format!("{base_url}/compatible-mode/v1"), "qwen3.7-flash"),
            ),
            &default_text_registry(),
        )
        .expect("text failures should preserve raw text");
        handle.join().expect("server result");

        assert_eq!(result.output.text, "保留原文");
        assert_eq!(
            result.attempts[0].error_kind,
            Some(ProviderErrorKind::InvalidResponse)
        );
    }
}

#[test]
#[ignore = "需要 DASHSCOPE_API_KEY 与 XILUOLIN_QWEN_ASR_SAMPLE 才会调用真实千问服务"]
fn qwen_audio_real_smoke_reads_key_and_sample_only_from_environment() {
    let api_key = std::env::var("DASHSCOPE_API_KEY").expect("DASHSCOPE_API_KEY");
    let audio_path =
        PathBuf::from(std::env::var("XILUOLIN_QWEN_ASR_SAMPLE").expect("XILUOLIN_QWEN_ASR_SAMPLE"));
    let mut provider_settings = settings(
        "https://dashscope.aliyuncs.com".to_string(),
        "qwen-audio-3.0-asr-flash",
    );
    provider_settings.api_key = api_key;
    let input = AsrInput {
        audio_path,
        hotwords: Vec::new(),
        context_prompt: None,
        local_model_path: None,
    };

    let result = route_asr(
        &input,
        &route("qwen-audio", provider_settings),
        &default_asr_registry(),
    );

    assert!(result.is_ok(), "真实千问转写应成功：{result:?}");
}
