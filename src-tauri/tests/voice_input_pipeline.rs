use xiluolin_lib::pipeline::{prepare_uploaded_audio_file, VoiceInputError};

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

    assert!(path.exists());
    assert_eq!(
        path.extension().and_then(|value| value.to_str()),
        Some("mp3")
    );

    std::fs::remove_file(path).expect("temporary uploaded audio should be removed");
}
