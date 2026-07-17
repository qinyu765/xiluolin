use std::path::Path;

#[test]
#[ignore = "requires an external whisper.cpp model and WAV fixture"]
fn transcribes_with_real_local_model() {
    let model = std::env::var("XILUOLIN_LOCAL_ASR_MODEL")
        .expect("XILUOLIN_LOCAL_ASR_MODEL must point to a whisper.cpp model");
    let audio = std::env::var("XILUOLIN_LOCAL_ASR_AUDIO")
        .expect("XILUOLIN_LOCAL_ASR_AUDIO must point to a WAV fixture");

    let text = xiluolin_lib::local_asr::transcribe(Path::new(&audio), Path::new(&model))
        .expect("local transcription should pass");
    assert!(!text.trim().is_empty());
}
