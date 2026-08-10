use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use xiluolin_lib::{
    asr::AsrConfig,
    data::HotwordDraft,
    history_reprocessing::persist_reprocessed_history,
    pipeline::{
        normalize_verbatim_text, prepare_uploaded_audio_file, process_voice_input_with_progress,
        HistoryContext, VoiceInputError, VoiceInputRequest, VoiceInputStage,
    },
    text_polish::TextPolishConfig,
};

mod common;

use common::{open_test_database, temp_db_path};

#[test]
fn rejects_empty_uploaded_audio_before_provider_request() {
    let error = prepare_uploaded_audio_file(Vec::new(), "wav")
        .expect_err("empty uploaded audio should fail before provider request");

    assert_eq!(error, VoiceInputError::EmptyAudio);
}

#[test]
fn writes_uploaded_audio_to_temporary_file_with_safe_extension() {
    let path = prepare_uploaded_audio_file(b"fixture audio".to_vec(), ".MP3")
        .expect("uploaded mp3 should be written to a temporary file");

    assert!(path.path().exists());
    assert_eq!(
        path.path().extension().and_then(|value| value.to_str()),
        Some("mp3")
    );

    drop(path);
}

#[test]
fn verbatim_normalization_only_trims_and_collapses_unicode_whitespace() {
    assert_eq!(
        normalize_verbatim_text("\u{3000} Hello,\t\nWORLD！ \u{00a0}"),
        "Hello, WORLD！"
    );
}

#[test]
fn verbatim_pipeline_sends_hotwords_to_asr_without_persona_prompt_or_text_provider() {
    let database = open_test_database(&temp_db_path("verbatim-pipeline"));
    database
        .set_default_persona("verbatim")
        .expect("verbatim persona should become default");
    let delete_error = database
        .delete_persona("verbatim")
        .expect_err("the active verbatim persona should not be deletable");
    assert!(delete_error.contains("默认人格不可删除"));
    for text in ["  XiLuoLin ", "智谱", "XiLuoLin"] {
        database
            .create_hotword(HotwordDraft {
                text: text.to_string(),
                category: "".to_string(),
                enabled: true,
            })
            .expect("hotword should be created");
    }
    let (base_url, handle) = spawn_mock_asr_server(r#"{"text":"  Hello,\t\nWORLD！  "}"#);
    let mut stages = Vec::new();

    let result = process_voice_input_with_progress(
        VoiceInputRequest {
            audio_bytes: b"fixture audio".to_vec(),
            audio_extension: "wav".to_string(),
            duration_ms: 600,
        },
        zhipu_asr_config(base_url),
        TextPolishConfig {
            provider: "openai".to_string(),
            api_key: String::new(),
            base_url: "http://127.0.0.1:9".to_string(),
            model: "gpt-4o-mini".to_string(),
        },
        &database,
        true,
        HistoryContext {
            source: "test".to_string(),
            text_provider: "openai".to_string(),
            text_model: "gpt-4o-mini".to_string(),
            audio_path: None,
        },
        |stage| stages.push(stage),
    )
    .expect("verbatim mode should not require a text provider");
    let request_bytes = handle.join().expect("ASR server should finish");
    let request = String::from_utf8_lossy(&request_bytes);

    assert_eq!(result.final_text, "Hello, WORLD！");
    assert!(!result.used_text_fallback);
    assert_eq!(stages, vec![VoiceInputStage::Transcribing]);
    assert!(request.contains("name=\"hotwords[]\"\r\n\r\nXiLuoLin"));
    assert!(request.contains("name=\"hotwords[]\"\r\n\r\n智谱"));
    assert!(!request.contains("保留语音识别原文"));
    let history = result
        .history_record
        .as_ref()
        .expect("history should be saved");
    assert_eq!(history.text_processing_mode, "verbatim");
    assert_eq!(history.text_provider, "");
    assert_eq!(history.text_model, "");
    assert_eq!(result.actual_text_provider, "");
    assert_eq!(result.actual_text_model, "");
    assert_eq!(result.text_processing_mode, "verbatim");

    let reprocessed = persist_reprocessed_history(&database, &history.id, &result)
        .expect("verbatim reprocessing should preserve actual processing metadata");
    assert_eq!(reprocessed.text_processing_mode, "verbatim");
    assert_eq!(reprocessed.text_provider, "");
    assert_eq!(reprocessed.text_model, "");
}

#[test]
fn missing_default_persona_cleans_up_uploaded_audio_before_asr() {
    let database = open_test_database(&temp_db_path("pipeline-missing-default"));
    let connection = rusqlite::Connection::open(database.path())
        .expect("database connection should open for test setup");
    connection
        .execute("UPDATE personas SET is_default = 0", [])
        .expect("test setup should clear default persona");
    let private_audio = format!(
        "private fixture audio {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    )
    .into_bytes();

    let error = process_voice_input_with_progress(
        VoiceInputRequest {
            audio_bytes: private_audio.clone(),
            audio_extension: "wav".to_string(),
            duration_ms: 100,
        },
        zhipu_asr_config("http://127.0.0.1:9".to_string()),
        TextPolishConfig {
            provider: "openai".to_string(),
            api_key: String::new(),
            base_url: String::new(),
            model: String::new(),
        },
        &database,
        false,
        HistoryContext {
            source: "test".to_string(),
            text_provider: String::new(),
            text_model: String::new(),
            audio_path: None,
        },
        |_| {},
    )
    .expect_err("missing default persona should stop before ASR");

    assert_eq!(error, VoiceInputError::MissingDefaultPersona);
    assert!(!temporary_audio_contents().contains(&private_audio));
}

#[test]
fn polish_pipeline_keeps_all_hotwords_for_asr_and_text_processing() {
    let database = open_test_database(&temp_db_path("polish-pipeline"));
    for index in 0..101 {
        database
            .create_hotword(HotwordDraft {
                text: format!("词{index}"),
                category: "".to_string(),
                enabled: true,
            })
            .expect("hotword should be created");
    }
    let (asr_base_url, asr_handle) = spawn_mock_asr_server(r#"{"text":"原始识别文本"}"#);
    let (text_base_url, text_handle) = spawn_mock_asr_server(
        r#"{"choices":[{"message":{"role":"assistant","content":"润色结果"}}]}"#,
    );
    let mut stages = Vec::new();

    let result = process_voice_input_with_progress(
        VoiceInputRequest {
            audio_bytes: b"fixture audio".to_vec(),
            audio_extension: "wav".to_string(),
            duration_ms: 600,
        },
        openai_asr_config(asr_base_url),
        TextPolishConfig {
            provider: "openai".to_string(),
            api_key: "text-key".to_string(),
            base_url: text_base_url,
            model: "gpt-4o-mini".to_string(),
        },
        &database,
        true,
        HistoryContext {
            source: "test".to_string(),
            text_provider: "openai".to_string(),
            text_model: "gpt-4o-mini".to_string(),
            audio_path: None,
        },
        |stage| stages.push(stage),
    )
    .expect("polish mode should process the transcription");
    let asr_request =
        String::from_utf8_lossy(&asr_handle.join().expect("ASR server should finish")).into_owned();
    let text_request =
        String::from_utf8_lossy(&text_handle.join().expect("text server should finish"))
            .into_owned();

    assert_eq!(result.final_text, "润色结果");
    assert_eq!(
        stages,
        vec![VoiceInputStage::Transcribing, VoiceInputStage::Refining]
    );
    assert!(asr_request.contains("词100"));
    assert!(!asr_request.contains("让文本保持自然、清晰"));
    assert!(text_request.contains("词100"));
    assert!(text_request.contains("让文本保持自然、清晰"));
    assert_eq!(
        result
            .history_record
            .expect("history should be saved")
            .text_processing_mode,
        "polish"
    );
}

fn zhipu_asr_config(base_url: String) -> AsrConfig {
    AsrConfig {
        provider: "zhipu".to_string(),
        api_key: "test-key".to_string(),
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

fn openai_asr_config(base_url: String) -> AsrConfig {
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
                let complete = String::from_utf8_lossy(&request)
                    .split_once("\r\n\r\n")
                    .and_then(|(headers, _)| {
                        headers
                            .lines()
                            .find_map(|line| line.strip_prefix("Content-Length: "))
                            .and_then(|value| value.parse::<usize>().ok())
                            .map(|length| headers.len() + 4 + length)
                    })
                    .is_some_and(|length| request.len() >= length);
                if complete {
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

fn temporary_audio_contents() -> Vec<Vec<u8>> {
    let directory = std::env::temp_dir().join("xiluolin-audio");
    std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| std::fs::read(entry.path()).ok())
        .collect()
}
