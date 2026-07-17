use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use xiluolin_lib::{
    asr::{transcribe_audio_file, AsrConfig, AsrError},
    data::default_app_config,
};

fn temp_audio_path(test_name: &str, extension: &str, bytes: &[u8]) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir()
        .join("xiluolin-asr-tests")
        .join(format!("{test_name}-{nanos}.{extension}"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test audio parent should be created");
    }
    fs::write(&path, bytes).expect("test audio should be written");
    path
}

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
                if request.windows(4).any(|window| window == b"\r\n\r\n")
                    && String::from_utf8_lossy(&request).contains("fixture audio")
                {
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

fn spawn_mock_asr_server(response_body: &'static str) -> (String, thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    let base_url = format!(
        "http://{}",
        listener
            .local_addr()
            .expect("mock server address should be readable")
    );

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("mock server should accept request");
        let request = read_request(&mut stream);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
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

fn asr_config(base_url: String, api_key: &str) -> AsrConfig {
    AsrConfig {
        provider: "zhipu".to_string(),
        api_key: api_key.to_string(),
        base_url,
        model: "glm-asr-2512".to_string(),
        local_model_path: None,
        allow_cloud_fallback: false,
        fallback_provider: String::new(),
        fallback_api_key: String::new(),
        fallback_base_url: String::new(),
        fallback_model: String::new(),
    }
}

#[test]
fn default_config_uses_zhipu_asr_endpoint_and_model() {
    let config = default_app_config();

    assert_eq!(config.asr_base_url, "https://open.bigmodel.cn/api/paas/v4");
    assert_eq!(config.asr_model, "glm-asr-2512");
}

#[test]
fn rejects_missing_api_key_before_network_request() {
    let audio_path = temp_audio_path("missing-key", "wav", b"fixture audio");
    let error = transcribe_audio_file(
        Path::new(&audio_path),
        &asr_config("http://127.0.0.1:9".to_string(), ""),
    )
    .expect_err("missing api key should fail");

    assert_eq!(error, AsrError::MissingApiKey);
}

#[test]
fn rejects_unsupported_audio_extension() {
    let audio_path = temp_audio_path("unsupported-extension", "txt", b"fixture audio");
    let error = transcribe_audio_file(
        Path::new(&audio_path),
        &asr_config("http://127.0.0.1:9".to_string(), "test-key"),
    )
    .expect_err("unsupported extension should fail");

    assert_eq!(error, AsrError::UnsupportedAudioFormat("txt".to_string()));
}

#[test]
fn posts_audio_to_zhipu_transcriptions_endpoint() {
    let audio_path = temp_audio_path("success", "wav", b"fixture audio");
    let (base_url, handle) = spawn_mock_asr_server(r#"{"text":"整理后的原始识别文本"}"#);

    let result = transcribe_audio_file(
        Path::new(&audio_path),
        &asr_config(format!("{base_url}/api/paas/v4/"), "test-key"),
    )
    .expect("mock transcribe should pass");
    let request = handle.join().expect("mock server should finish");
    let request_text = String::from_utf8_lossy(&request);
    let request_lowercase = request_text.to_ascii_lowercase();

    assert_eq!(result.text, "整理后的原始识别文本");
    assert!(request_text.starts_with("POST /api/paas/v4/audio/transcriptions HTTP/1.1"));
    assert!(request_lowercase.contains("authorization: bearer test-key"));
    assert!(request_lowercase.contains("content-type: multipart/form-data; boundary="));
    assert!(request_text.contains("name=\"model\""));
    assert!(request_text.contains("glm-asr-2512"));
    assert!(request_text.contains("name=\"file\""));
    assert!(request_text.contains("filename=\""));
    assert!(request_text.contains("fixture audio"));
}
