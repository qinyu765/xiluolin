use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use xiluolin_lib::{
    data::default_app_config,
    text_polish::{
        polish_text_with_provider, TextPolishConfig, TextPolishError, TextPolishRequest,
    },
};

fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .expect("read timeout should be set");

    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                request.extend_from_slice(&buffer[..count]);
                if let Some(header_end) =
                    request.windows(4).position(|window| window == b"\r\n\r\n")
                {
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
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(error) => panic!("request should be readable: {error}"),
        }
    }

    request
}

fn spawn_mock_openai_server(
    response_status: &str,
    response_body: &str,
) -> (String, thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    let base_url = format!(
        "http://{}",
        listener
            .local_addr()
            .expect("mock server address should be readable")
    );

    let response_status = response_status.to_string();
    let response_body = response_body.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("mock server should accept request");
        let request = read_request(&mut stream);
        let response = format!(
            "HTTP/1.1 {response_status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("mock response should be written");
        request
    });

    (base_url, handle)
}

fn openai_config(base_url: String, api_key: &str) -> TextPolishConfig {
    TextPolishConfig {
        provider: "openai".to_string(),
        api_key: api_key.to_string(),
        base_url,
        model: "gpt-4o-mini".to_string(),
    }
}

fn zhipu_config(base_url: String, api_key: &str) -> TextPolishConfig {
    TextPolishConfig {
        provider: "zhipu".to_string(),
        api_key: api_key.to_string(),
        base_url,
        model: "glm-4.7".to_string(),
    }
}

fn polish_request() -> TextPolishRequest {
    TextPolishRequest {
        persona_id: "prompt-engineer".to_string(),
        raw_text: "这个任务帮我整理一下原始识别文本".to_string(),
        persona_description: "你是 Prompt 工程师，请整理成可执行 Prompt。".to_string(),
        hotword_context: "- codex -> Codex（工具名）".to_string(),
    }
}

#[test]
fn default_config_contains_text_provider_and_zhipu_config() {
    let config = default_app_config();

    assert_eq!(config.text_provider, "zhipu");
    assert_eq!(config.zhipu_api_key, "");
    assert_eq!(
        config.zhipu_base_url,
        "https://open.bigmodel.cn/api/paas/v4"
    );
    assert_eq!(config.zhipu_model, "glm-4.7-flash");
    assert_eq!(config.openai_api_key, "");
    assert_eq!(config.openai_base_url, "https://api.openai.com/v1");
    assert_eq!(config.openai_model, "gpt-4o-mini");
}

#[test]
fn rejects_missing_openai_api_key_before_network_request() {
    let error = polish_text_with_provider(
        &polish_request(),
        &openai_config("http://127.0.0.1:9".to_string(), ""),
    )
    .expect_err("missing api key should fail");

    assert_eq!(error, TextPolishError::MissingApiKey);
}

#[test]
fn posts_persona_hotwords_and_raw_text_to_chat_completions_endpoint() {
    let (base_url, handle) = spawn_mock_openai_server(
        "200 OK",
        r#"{"choices":[{"message":{"role":"assistant","content":"请帮我整理这个任务：原始识别文本。"}}]}"#,
    );

    let result = polish_text_with_provider(
        &polish_request(),
        &openai_config(format!("{base_url}/v1"), "test-key"),
    )
    .expect("mock polish should pass");
    let request = handle.join().expect("mock server should finish");
    let request_text = String::from_utf8_lossy(&request);
    let request_lowercase = request_text.to_ascii_lowercase();

    assert_eq!(result.final_text, "请帮我整理这个任务：原始识别文本。");
    assert!(!result.used_fallback);
    assert!(request_text.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(request_lowercase.contains("authorization: bearer test-key"));
    assert!(request_lowercase.contains("content-type: application/json"));
    assert!(request_text.contains(r#""model": "gpt-4o-mini""#));
    assert!(request_text.contains(r#""max_tokens": 512"#));
    assert!(!request_text.contains(r#""thinking""#));
    assert!(request_text.contains(r#""role": "system""#));
    assert!(request_text.contains("Prompt 工程师"));
    assert!(request_text.contains("保留用户原意"));
    assert!(request_text.contains("原始识别文本"));
    assert!(request_text.contains("codex -> Codex"));
}

#[test]
fn zhipu_requests_disable_thinking_by_default() {
    let (base_url, handle) = spawn_mock_openai_server(
        "200 OK",
        r#"{"choices":[{"message":{"role":"assistant","content":"整理结果"}}]}"#,
    );

    let result = polish_text_with_provider(
        &polish_request(),
        &zhipu_config(format!("{base_url}/api/paas/v4"), "test-key"),
    )
    .expect("mock polish should pass");
    let raw_request = handle.join().expect("mock server should finish");
    let request_text = String::from_utf8_lossy(&raw_request);

    assert_eq!(result.final_text, "整理结果");
    let body = request_text
        .split_once("\r\n\r\n")
        .expect("request should contain a body")
        .1;
    let body: serde_json::Value = serde_json::from_str(body).expect("body should be valid JSON");
    assert_eq!(body["thinking"]["type"], "disabled");
    assert_eq!(body["max_tokens"], 512);
}

#[test]
fn request_requires_persona_style_rewrite_instead_of_plain_cleanup() {
    let (base_url, handle) = spawn_mock_openai_server(
        "200 OK",
        r#"{"choices":[{"message":{"role":"assistant","content":"今天我以能量补给赋能为前置抓手。"}}]}"#,
    );
    let request = TextPolishRequest {
        persona_id: "prompt-engineer".to_string(),
        raw_text: "今天中午我吃了饭，买了杯咖啡，下午和同事聊了聊需求。".to_string(),
        persona_description: "黑话大师：将语音转化为更互联网黑话的形式。".to_string(),
        hotword_context: String::new(),
    };

    let result = polish_text_with_provider(
        &request,
        &openai_config(format!("{base_url}/v1"), "test-key"),
    )
    .expect("mock polish should pass");
    let raw_request = handle.join().expect("mock server should finish");
    let request_text = String::from_utf8_lossy(&raw_request);

    assert_eq!(result.final_text, "今天我以能量补给赋能为前置抓手。");
    assert!(request_text.contains("黑话大师"));
    assert!(request_text.contains("风格化改写"));
    assert!(request_text.contains("不要原样返回"));
    assert!(request_text.contains("只做标点或语病清理"));
}

#[test]
fn general_persona_uses_light_cleanup_prompt_and_removes_terminal_period() {
    let (base_url, handle) = spawn_mock_openai_server(
        "200 OK",
        r#"{"choices":[{"message":{"role":"assistant","content":"今天把需求聊清楚了。晚点再整理一下。"}}]}"#,
    );
    let request = TextPolishRequest {
        persona_id: "general".to_string(),
        raw_text: "嗯今天把需求聊清楚了然后晚点再整理一下".to_string(),
        persona_description:
            "让文本保持自然、清晰、口语化的语气，同时更精炼易读，要把句尾的句号去掉。".to_string(),
        hotword_context: "- Codex（工具名）".to_string(),
    };

    let result = polish_text_with_provider(
        &request,
        &openai_config(format!("{base_url}/v1"), "test-key"),
    )
    .expect("general persona polish should pass");
    let raw_request = handle.join().expect("mock server should finish");
    let request_text = String::from_utf8_lossy(&raw_request);

    assert_eq!(result.final_text, "今天把需求聊清楚了。晚点再整理一下");
    assert!(request_text.contains("轻量整理"));
    assert!(request_text.contains("只做最小修改"));
    assert!(request_text.contains("不要强制生成标题、列表、模板"));
    assert!(request_text.contains("Codex"));
    assert!(!request_text.contains("必须按人格风格要求进行风格化改写"));
}

#[test]
fn general_persona_applies_terminal_punctuation_rules() {
    for (content, expected) in [
        ("Ready.", "Ready"),
        ("要现在处理吗？", "要现在处理吗？"),
        ("马上开始！", "马上开始！"),
        ("我再想想...", "我再想想..."),
    ] {
        let response_body = format!(
            r#"{{"choices":[{{"message":{{"role":"assistant","content":"{content}"}}}}]}}"#
        );
        let (base_url, _handle) = spawn_mock_openai_server("200 OK", &response_body);
        let request = TextPolishRequest {
            persona_id: "general".to_string(),
            raw_text: content.to_string(),
            persona_description: "保持自然清晰".to_string(),
            hotword_context: String::new(),
        };

        let result = polish_text_with_provider(
            &request,
            &openai_config(format!("{base_url}/v1"), "test-key"),
        )
        .expect("general persona punctuation should pass");

        assert_eq!(result.final_text, expected);
    }
}

#[test]
fn general_persona_fallback_still_removes_terminal_period() {
    let (base_url, _handle) = spawn_mock_openai_server(
        "500 Internal Server Error",
        r#"{"error":{"message":"server error"}}"#,
    );
    let request = TextPolishRequest {
        persona_id: "general".to_string(),
        raw_text: "保留原始文本。".to_string(),
        persona_description: "保持自然清晰".to_string(),
        hotword_context: String::new(),
    };

    let result = polish_text_with_provider(
        &request,
        &openai_config(format!("{base_url}/v1"), "test-key"),
    )
    .expect("general persona fallback should pass");

    assert_eq!(result.final_text, "保留原始文本");
    assert!(result.used_fallback);
}

#[test]
fn request_failure_returns_raw_text_as_fallback() {
    let (base_url, _handle) = spawn_mock_openai_server(
        "500 Internal Server Error",
        r#"{"error":{"message":"server error"}}"#,
    );

    let result = polish_text_with_provider(
        &polish_request(),
        &openai_config(format!("{base_url}/v1/"), "test-key"),
    )
    .expect("request failure should keep raw text fallback");

    assert_eq!(result.final_text, polish_request().raw_text);
    assert!(result.used_fallback);
    assert!(result
        .error_message
        .expect("fallback should keep error message")
        .contains("文本整理请求失败"));
}

#[test]
fn prompt_leak_response_is_rejected_before_history_can_store_it() {
    let (base_url, _handle) = spawn_mock_openai_server(
        "200 OK",
        r#"{"choices":[{"message":{"role":"assistant","content":"目标：将语音识别的原始文本转化为结构清晰的 Prompt。上下文：用户提供了 ASR 原始识别文本，需要按人格风格改写。约束：必须严格遵循指定的人格风格。"}}]}"#,
    );
    let request = polish_request();

    let result = polish_text_with_provider(
        &request,
        &zhipu_config(format!("{base_url}/api/paas/v4"), "test-key"),
    )
    .expect("prompt leak should use the safe raw-text fallback");

    assert_eq!(result.final_text, request.raw_text);
    assert!(result.used_fallback);
    assert!(result
        .error_message
        .expect("fallback should explain why the response was rejected")
        .contains("内部整理指令"));
}

#[test]
fn structured_prompt_output_is_kept_when_it_only_uses_user_content() {
    let (base_url, _handle) = spawn_mock_openai_server(
        "200 OK",
        r#"{"choices":[{"message":{"role":"assistant","content":"目标：比较两个包的价格差异。\n上下文：一个包价格更低，另一个包价格更高。\n约束：仅依据给出的价格信息。\n期望结果：给出简洁的购买建议。"}}]}"#,
    );
    let request = TextPolishRequest {
        persona_id: "prompt-engineer".to_string(),
        raw_text: "帮我比较一下这两个包的价格，一个便宜一点，一个贵一点，然后给我一个购买建议。"
            .to_string(),
        persona_description:
            "将语音转换为清晰、可执行的 AI Prompt。输出结构：目标、上下文、约束、期望结果。"
                .to_string(),
        hotword_context: String::new(),
    };

    let result = polish_text_with_provider(
        &request,
        &zhipu_config(format!("{base_url}/api/paas/v4"), "test-key"),
    )
    .expect("valid prompt output should pass");

    assert!(!result.used_fallback);
    assert!(result.final_text.contains("目标：比较两个包的价格差异"));
    assert!(result.final_text.contains("期望结果：给出简洁的购买建议"));
}

#[test]
fn prompt_marker_spoken_by_user_is_not_treated_as_internal_leak() {
    let (base_url, _handle) = spawn_mock_openai_server(
        "200 OK",
        r#"{"choices":[{"message":{"role":"assistant","content":"请把“人格风格要求”这个字段改成“表达风格”。"}}]}"#,
    );
    let request = TextPolishRequest {
        persona_id: "prompt-engineer".to_string(),
        raw_text: "请把人格风格要求这个字段改成表达风格。".to_string(),
        persona_description: "用简洁的产品说明表达。".to_string(),
        hotword_context: String::new(),
    };

    let result = polish_text_with_provider(
        &request,
        &openai_config(format!("{base_url}/v1"), "test-key"),
    )
    .expect("user-spoken marker should remain valid");

    assert!(!result.used_fallback);
    assert!(result.final_text.contains("人格风格要求"));
}
