use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use xiluolin_lib::{
    data::default_app_config,
    text_polish::{
        polish_text_with_openai, TextPolishConfig, TextPolishError, TextPolishRequest,
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
                if String::from_utf8_lossy(&request).contains("原始识别文本") {
                    break;
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
    response_status: &'static str,
    response_body: &'static str,
) -> (String, thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    let base_url = format!(
        "http://{}",
        listener
            .local_addr()
            .expect("mock server address should be readable")
    );

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("mock server should accept request");
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

fn polish_request() -> TextPolishRequest {
    TextPolishRequest {
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
    assert_eq!(config.zhipu_base_url, "https://open.bigmodel.cn/api/paas/v4");
    assert_eq!(config.zhipu_model, "glm-4.7-flash");
    assert_eq!(config.openai_api_key, "");
    assert_eq!(config.openai_base_url, "https://api.openai.com/v1");
    assert_eq!(config.openai_model, "gpt-4o-mini");
}

#[test]
fn rejects_missing_openai_api_key_before_network_request() {
    let error = polish_text_with_openai(
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

    let result = polish_text_with_openai(
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
    assert!(request_text.contains(r#""role": "system""#));
    assert!(request_text.contains("Prompt 工程师"));
    assert!(request_text.contains("保留用户原意"));
    assert!(request_text.contains("原始识别文本"));
    assert!(request_text.contains("codex -> Codex"));
}

#[test]
fn request_failure_returns_raw_text_as_fallback() {
    let (base_url, _handle) = spawn_mock_openai_server(
        "500 Internal Server Error",
        r#"{"error":{"message":"server error"}}"#,
    );

    let result = polish_text_with_openai(
        &polish_request(),
        &openai_config(format!("{base_url}/v1/"), "test-key"),
    )
    .expect("request failure should keep raw text fallback");

    assert_eq!(result.final_text, polish_request().raw_text);
    assert!(result.used_fallback);
    assert!(
        result
            .error_message
            .expect("fallback should keep error message")
            .contains("OpenAI 文本整理请求失败")
    );
}
