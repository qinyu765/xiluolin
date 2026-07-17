use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use xiluolin_lib::pipeline::consume_app_recording;

fn temp_dir(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    std::env::temp_dir()
        .join("xiluolin-recording-security-tests")
        .join(format!("{test_name}-{nanos}"))
}

fn create_recording(test_name: &str) -> (PathBuf, PathBuf) {
    let root = temp_dir(test_name);
    let recordings = root.join("recordings");
    fs::create_dir_all(&recordings).expect("recordings directory should be created");
    let recording = recordings.join("recording.wav");
    fs::write(&recording, b"test-audio").expect("recording should be created");
    (recordings, recording)
}

#[test]
fn successful_processing_deletes_recording() {
    let (recordings, recording) = create_recording("success-cleanup");

    let result = consume_app_recording(&recordings, &recording, |bytes, extension, _path| {
        assert_eq!(bytes, b"test-audio");
        assert_eq!(extension, "wav");
        Ok(("processed", false))
    })
    .expect("processing should pass");

    assert_eq!(result, "processed");
    assert!(!recording.exists());
}

#[test]
fn failed_processing_still_deletes_recording() {
    let (recordings, recording) = create_recording("failure-cleanup");

    let result =
        consume_app_recording::<()>(&recordings, &recording, |_bytes, _extension, _path| {
            Err("模拟配置或 ASR 失败".to_string())
        });

    assert_eq!(result.unwrap_err(), "模拟配置或 ASR 失败");
    assert!(!recording.exists());
}

#[test]
fn successful_processing_can_retain_recording() {
    let (recordings, recording) = create_recording("retain-success");

    let result = consume_app_recording(&recordings, &recording, |_bytes, _extension, path| {
        assert_eq!(
            path,
            recording
                .canonicalize()
                .expect("recording should canonicalize")
        );
        Ok(("retained", true))
    })
    .expect("processing should pass");

    assert_eq!(result, "retained");
    assert!(recording.exists());
}

#[test]
fn panicking_processing_still_deletes_recording() {
    let (recordings, recording) = create_recording("panic-cleanup");

    let result = std::panic::catch_unwind(|| {
        let _ =
            consume_app_recording::<()>(&recordings, &recording, |_bytes, _extension, _path| {
                panic!("模拟处理 panic")
            });
    });

    assert!(result.is_err());
    assert!(!recording.exists());
}

#[test]
fn external_file_is_rejected_and_not_deleted() {
    let root = temp_dir("external-file");
    let recordings = root.join("recordings");
    fs::create_dir_all(&recordings).expect("recordings directory should be created");
    let external = root.join("external.wav");
    fs::write(&external, b"user-audio").expect("external file should be created");

    let result =
        consume_app_recording::<()>(&recordings, &external, |_bytes, _extension, _path| {
            panic!("external files must not be processed")
        });

    assert_eq!(result.unwrap_err(), "录音文件不在应用录音目录中");
    assert!(external.exists());
}

#[test]
fn traversal_path_is_rejected_and_not_deleted() {
    let root = temp_dir("traversal");
    let recordings = root.join("recordings");
    fs::create_dir_all(&recordings).expect("recordings directory should be created");
    let external = root.join("outside.wav");
    fs::write(&external, b"user-audio").expect("external file should be created");
    let traversal = recordings.join("..").join("outside.wav");

    let result =
        consume_app_recording::<()>(&recordings, &traversal, |_bytes, _extension, _path| {
            panic!("traversal path must not be processed")
        });

    assert_eq!(result.unwrap_err(), "录音文件不在应用录音目录中");
    assert!(external.exists());
}

#[test]
fn non_wav_file_inside_recordings_is_rejected_without_deletion() {
    let root = temp_dir("non-wav");
    let recordings = root.join("recordings");
    fs::create_dir_all(&recordings).expect("recordings directory should be created");
    let recording = recordings.join("recording.mp3");
    fs::write(&recording, b"audio").expect("file should be created");

    let result =
        consume_app_recording::<()>(&recordings, &recording, |_bytes, _extension, _path| {
            panic!("non-WAV file must not be processed")
        });

    assert_eq!(result.unwrap_err(), "应用录音文件必须是 WAV 格式");
    assert!(recording.exists());
}
