use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use xiluolin_lib::asr::{transcribe_audio_file, AsrConfig};

fn temp_path(extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("xiluolin-local-asr-{nanos}.{extension}"))
}

fn local_config(model_path: PathBuf) -> AsrConfig {
    AsrConfig {
        provider: "local".to_string(),
        api_key: String::new(),
        base_url: String::new(),
        model: "ggml-base-q5_1.bin".to_string(),
        local_model_path: Some(model_path),
        allow_cloud_fallback: false,
        fallback_provider: "zhipu".to_string(),
        fallback_api_key: String::new(),
        fallback_base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
        fallback_model: "glm-asr-2512".to_string(),
    }
}

#[test]
fn local_provider_requires_downloaded_model_without_cloud_request() {
    let audio = temp_path("wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&audio, spec).unwrap();
    writer.write_sample(0_i16).unwrap();
    writer.finalize().unwrap();

    let error = transcribe_audio_file(&audio, &local_config(temp_path("bin"))).unwrap_err();
    assert!(error.to_string().contains("模型尚未下载"));
    fs::remove_file(audio).unwrap();
}

#[test]
fn local_provider_rejects_mp3_before_inference() {
    let audio = temp_path("mp3");
    fs::write(&audio, b"not-an-mp3").unwrap();

    let error = transcribe_audio_file(&audio, &local_config(temp_path("bin"))).unwrap_err();
    assert!(error.to_string().contains("本地 ASR 首版仅支持 WAV"));
    fs::remove_file(audio).unwrap();
}

#[test]
fn cloud_fallback_runs_only_when_explicitly_enabled() {
    let audio = temp_path("wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&audio, spec).unwrap();
    writer.write_sample(0_i16).unwrap();
    writer.finalize().unwrap();

    let mut config = local_config(temp_path("bin"));
    config.allow_cloud_fallback = true;
    let error = transcribe_audio_file(&audio, &config).unwrap_err();
    assert!(error.to_string().contains("API Key"));
    fs::remove_file(audio).unwrap();
}
