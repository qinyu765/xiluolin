use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use xiluolin_lib::{
    asr::{
        build_soft_prompt, transcribe_audio_file, AsrCapabilities, AsrConfig, AsrError, AsrRequest,
    },
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
    let mut expected_len = None;
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                request.extend_from_slice(&buffer[..count]);
                if expected_len.is_none() {
                    let request_text = String::from_utf8_lossy(&request);
                    if let Some((headers, _)) = request_text.split_once("\r\n\r\n") {
                        expected_len = headers.lines().find_map(|line| {
                            line.strip_prefix("Content-Length: ")
                                .or_else(|| line.strip_prefix("content-length: "))
                                .and_then(|value| value.parse::<usize>().ok())
                                .map(|length| headers.len() + 4 + length)
                        });
                    }
                }
                if expected_len.is_some_and(|length| request.len() >= length) {
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

fn spawn_mock_asr_server_with_status(
    status: &'static str,
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
        let (mut stream, _) = listener
            .accept()
            .expect("mock server should accept request");
        let request = read_request(&mut stream);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
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

fn spawn_mock_asr_server(response_body: &'static str) -> (String, thread::JoinHandle<Vec<u8>>) {
    spawn_mock_asr_server_with_status("200 OK", response_body)
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

fn openai_config(base_url: String) -> AsrConfig {
    AsrConfig {
        provider: "openai".to_string(),
        api_key: "test-key".to_string(),
        base_url,
        model: "whisper-1".to_string(),
        local_model_path: None,
        allow_cloud_fallback: false,
        fallback_provider: String::new(),
        fallback_api_key: String::new(),
        fallback_base_url: String::new(),
        fallback_model: String::new(),
    }
}

fn request(audio_path: PathBuf) -> AsrRequest {
    AsrRequest {
        audio_path,
        hotwords: Vec::new(),
        context_prompt: None,
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
        &request(audio_path),
        &asr_config("http://127.0.0.1:9".to_string(), ""),
    )
    .expect_err("missing api key should fail");

    assert_eq!(error, AsrError::MissingApiKey);
}

#[test]
fn rejects_unsupported_audio_extension() {
    let audio_path = temp_audio_path("unsupported-extension", "txt", b"fixture audio");
    let error = transcribe_audio_file(
        &request(audio_path),
        &asr_config("http://127.0.0.1:9".to_string(), "test-key"),
    )
    .expect_err("unsupported extension should fail");

    assert_eq!(error, AsrError::UnsupportedAudioFormat("txt".to_string()));
}

#[test]
fn multipart_asr_preserves_http_status_for_provider_classification() {
    for (status, expected) in [
        ("401 Unauthorized", 401),
        ("429 Too Many Requests", 429),
        ("503 Service Unavailable", 503),
    ] {
        let audio_path = temp_audio_path("http-status", "wav", b"fixture audio");
        let (base_url, handle) = spawn_mock_asr_server_with_status(status, "{}");
        let error = transcribe_audio_file(
            &request(audio_path.clone()),
            &asr_config(base_url, "test-key"),
        )
        .expect_err("HTTP failure should be structured");
        handle.join().expect("mock server");
        fs::remove_file(audio_path).expect("remove fixture");

        assert_eq!(error, AsrError::HttpStatus(expected));
    }
}

#[test]
fn posts_audio_to_zhipu_transcriptions_endpoint() {
    let audio_path = temp_audio_path("success", "wav", b"fixture audio");
    let (base_url, handle) = spawn_mock_asr_server(r#"{"text":"整理后的原始识别文本"}"#);

    let result = transcribe_audio_file(
        &request(audio_path),
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
    assert!(request_text.contains("name=\"stream\""));
    assert!(request_text.contains("\r\nfalse\r\n"));
    assert!(request_text.contains("name=\"file\""));
    assert!(request_text.contains("filename=\""));
    assert!(request_text.contains("fixture audio"));
}

#[test]
fn zhipu_multipart_filters_and_stably_deduplicates_hotwords() {
    let audio_path = temp_audio_path("hotwords", "wav", b"fixture audio");
    let (base_url, handle) = spawn_mock_asr_server(r#"{"text":"热词识别文本"}"#);
    let mut hotwords = vec![
        "  XiLuoLin ".to_string(),
        "".to_string(),
        "智谱".to_string(),
        "XiLuoLin".to_string(),
        "   ".to_string(),
    ];
    hotwords.extend((0..120).map(|index| format!("词{index}")));

    transcribe_audio_file(
        &AsrRequest {
            audio_path,
            hotwords,
            context_prompt: Some("上一段转写".to_string()),
        },
        &asr_config(base_url, "test-key"),
    )
    .expect("mock transcribe should pass");
    let request = handle.join().expect("mock server should finish");
    let request_text = String::from_utf8_lossy(&request);

    assert!(request_text.contains("name=\"prompt\"\r\n\r\n上一段转写"));
    assert_eq!(request_text.matches("name=\"hotwords[]\"").count(), 100);
    assert!(request_text.contains("\r\nXiLuoLin\r\n"));
    assert!(request_text.contains("\r\n智谱\r\n"));
    assert!(request_text.contains("\r\n词97\r\n"));
    assert!(!request_text.contains("\r\n词98\r\n"));
}

#[test]
fn openai_multipart_sends_context_and_hotwords_as_prompt() {
    let audio_path = temp_audio_path("openai-prompt", "wav", b"fixture audio");
    let (base_url, handle) = spawn_mock_asr_server(r#"{"text":"提示词识别文本"}"#);

    transcribe_audio_file(
        &AsrRequest {
            audio_path,
            hotwords: vec![
                "  XiLuoLin ".to_string(),
                "智谱".to_string(),
                "XiLuoLin".to_string(),
                " ".to_string(),
            ],
            context_prompt: Some("  会议背景  ".to_string()),
        },
        &openai_config(base_url),
    )
    .expect("mock transcribe should pass");
    let request = handle.join().expect("mock server should finish");
    let request_text = String::from_utf8_lossy(&request);

    assert!(
        request_text.contains("name=\"prompt\"\r\n\r\n会议背景\n可能出现的专有词：XiLuoLin，智谱")
    );
}

#[test]
fn soft_prompt_uses_trimmed_context_and_stable_hotwords() {
    assert_eq!(
        build_soft_prompt(
            Some("  会议背景  "),
            &[
                "  XiLuoLin ".to_string(),
                "智谱".to_string(),
                "XiLuoLin".to_string()
            ]
        ),
        Some("会议背景\n可能出现的专有词：XiLuoLin，智谱".to_string())
    );
    assert_eq!(build_soft_prompt(Some("  "), &[]), None);
}

#[test]
fn provider_capabilities_match_supported_asr_features() {
    assert_eq!(
        asr_config("http://127.0.0.1".to_string(), "test-key").capabilities(),
        AsrCapabilities {
            native_hotwords: true,
            max_hotwords: Some(100),
            supports_prompt: true,
            max_duration_ms: Some(30_000),
            live_audio: false,
        }
    );
    assert_eq!(
        openai_config("http://127.0.0.1".to_string()).capabilities(),
        AsrCapabilities {
            native_hotwords: false,
            max_hotwords: None,
            supports_prompt: true,
            max_duration_ms: None,
            live_audio: false,
        }
    );
}

#[test]
#[ignore = "需要显式设置 XILUOLIN_ZHIPU_ASR_KEY 后才会调用真实智谱服务"]
fn zhipu_real_smoke_reads_key_only_from_environment() {
    let api_key = std::env::var("XILUOLIN_ZHIPU_ASR_KEY")
        .expect("运行此 smoke test 前需要设置 XILUOLIN_ZHIPU_ASR_KEY");
    let audio_path = temp_audio_path("zhipu-real-smoke", "wav", &valid_silent_wav());

    let result = transcribe_audio_file(
        &request(audio_path.clone()),
        &asr_config("https://open.bigmodel.cn/api/paas/v4".to_string(), &api_key),
    );
    let _ = fs::remove_file(audio_path);

    assert!(result.is_ok(), "真实智谱转写请求应成功");
}

fn valid_silent_wav() -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut bytes);
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(cursor, spec).expect("WAV writer should be created");
        for _ in 0..1_600 {
            writer
                .write_sample(0_i16)
                .expect("silent sample should be written");
        }
        writer.finalize().expect("WAV writer should finalize");
    }
    bytes
}
