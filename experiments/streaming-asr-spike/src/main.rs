use std::{
    path::Path,
    process::ExitCode,
    thread,
    time::{Duration, Instant},
};

use sherpa_onnx::{OnlineRecognizer, OnlineRecognizerConfig, Wave};
use xiluolin_streaming_asr_spike::{build_feed_plan, CliArgs, FeedAction, RevisionTracker};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("错误：{error}");
            eprintln!(
                "用法：xiluolin-streaming-asr-spike \
                 --encoder <path> --decoder <path> --joiner <path> \
                 --tokens <path> --wav <path> [--hotword <text>]... \
                 [--realtime] [--repeat <count>]"
            );
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = CliArgs::parse_from(std::env::args())?;
    for (flag, path) in [
        ("--encoder", &args.encoder),
        ("--decoder", &args.decoder),
        ("--joiner", &args.joiner),
        ("--tokens", &args.tokens),
        ("--wav", &args.wav),
    ] {
        if !Path::new(path).is_file() {
            return Err(format!("{flag} 指定的文件不存在：{path}"));
        }
    }

    let wave = Wave::read(&args.wav).ok_or_else(|| "无法读取单声道 WAV".to_string())?;
    let sample_rate = u32::try_from(wave.sample_rate())
        .map_err(|_| format!("WAV 采样率无效：{}", wave.sample_rate()))?;
    if sample_rate == 0 || wave.samples().is_empty() {
        return Err("WAV 不包含有效 PCM".to_string());
    }

    let mut config = OnlineRecognizerConfig::default();
    config.model_config.transducer.encoder = Some(args.encoder.clone());
    config.model_config.transducer.decoder = Some(args.decoder.clone());
    config.model_config.transducer.joiner = Some(args.joiner.clone());
    config.model_config.tokens = Some(args.tokens.clone());
    config.model_config.num_threads = 2;
    config.decoding_method = Some("modified_beam_search".to_string());
    config.max_active_paths = 4;
    config.enable_endpoint = true;
    config.rule1_min_trailing_silence = 2.4;
    config.rule2_min_trailing_silence = 1.2;
    config.rule3_min_utterance_length = 20.0;
    config.hotwords_score = 1.5;

    let load_started = Instant::now();
    let recognizer = OnlineRecognizer::create(&config)
        .ok_or_else(|| "创建 sherpa-onnx OnlineRecognizer 失败".to_string())?;
    let load_ms = load_started.elapsed().as_secs_f64() * 1_000.0;
    println!(
        "model_load_ms={load_ms:.2} sample_rate_hz={} samples={} duration_ms={:.2} hotwords={} realtime={} repeat={}",
        sample_rate,
        wave.samples().len(),
        wave.samples().len() as f64 * 1_000.0 / sample_rate as f64,
        args.hotwords.len(),
        args.realtime,
        args.repeat
    );

    let hotwords = args.hotwords.join("\n");
    for run_index in 1..=args.repeat {
        let stream = if hotwords.is_empty() {
            recognizer.create_stream()
        } else {
            recognizer.create_stream_with_hotwords(&hotwords)
        };
        let started = Instant::now();
        let mut tracker = RevisionTracker::default();
        let mut first_partial_ms = None;
        let mut previous_update_ms = None;
        let mut update_intervals_ms = Vec::new();
        let mut audio_samples_sent = 0_usize;

        for action in build_feed_plan(wave.samples().len(), sample_rate) {
            match action {
                FeedAction::Audio { start, end } => {
                    stream.accept_waveform(wave.sample_rate(), &wave.samples()[start..end]);
                    audio_samples_sent = end;
                    decode_available(
                        &recognizer,
                        &stream,
                        &mut tracker,
                        run_index,
                        started,
                        audio_samples_sent,
                        sample_rate,
                        &mut first_partial_ms,
                        &mut previous_update_ms,
                        &mut update_intervals_ms,
                    );
                    if args.realtime {
                        let chunk_duration =
                            Duration::from_secs_f64((end - start) as f64 / sample_rate as f64);
                        let target = Duration::from_secs_f64(end as f64 / sample_rate as f64);
                        let remaining = target.saturating_sub(started.elapsed());
                        thread::sleep(remaining.min(chunk_duration));
                    }
                }
                FeedAction::TailPadding { samples } => {
                    let padding = vec![0.0_f32; samples];
                    stream.accept_waveform(wave.sample_rate(), &padding);
                    decode_available(
                        &recognizer,
                        &stream,
                        &mut tracker,
                        run_index,
                        started,
                        audio_samples_sent,
                        sample_rate,
                        &mut first_partial_ms,
                        &mut previous_update_ms,
                        &mut update_intervals_ms,
                    );
                }
                FeedAction::InputFinished => {
                    stream.input_finished();
                    decode_available(
                        &recognizer,
                        &stream,
                        &mut tracker,
                        run_index,
                        started,
                        audio_samples_sent,
                        sample_rate,
                        &mut first_partial_ms,
                        &mut previous_update_ms,
                        &mut update_intervals_ms,
                    );
                }
            }
        }

        let elapsed_ms = started.elapsed().as_secs_f64() * 1_000.0;
        let max_update_interval_ms = update_intervals_ms.iter().copied().fold(0.0_f64, f64::max);
        println!(
            "run={run_index} summary elapsed_ms={elapsed_ms:.2} first_partial_ms={} updates={} max_update_interval_ms={max_update_interval_ms:.2}",
            first_partial_ms
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "none".to_string()),
            tracker.revision()
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn decode_available(
    recognizer: &OnlineRecognizer,
    stream: &sherpa_onnx::OnlineStream,
    tracker: &mut RevisionTracker,
    run_index: usize,
    started: Instant,
    audio_samples_sent: usize,
    sample_rate: u32,
    first_partial_ms: &mut Option<f64>,
    previous_update_ms: &mut Option<f64>,
    update_intervals_ms: &mut Vec<f64>,
) {
    while recognizer.is_ready(stream) {
        recognizer.decode(stream);
        let is_endpoint = recognizer.is_endpoint(stream);
        if let Some(result) = recognizer.get_result(stream) {
            if let Some(revision) = tracker.observe(&result.text) {
                let wall_ms = started.elapsed().as_secs_f64() * 1_000.0;
                let audio_ms = audio_samples_sent as f64 * 1_000.0 / sample_rate as f64;
                if first_partial_ms.is_none() {
                    *first_partial_ms = Some(wall_ms);
                }
                if let Some(previous) = previous_update_ms.replace(wall_ms) {
                    update_intervals_ms.push(wall_ms - previous);
                }
                println!(
                    "run={run_index} revision={revision} audio_ms={audio_ms:.2} wall_ms={wall_ms:.2} endpoint={is_endpoint} segment_final={} text={}",
                    result.is_final,
                    result.text
                );
            }
        }
        if is_endpoint {
            recognizer.reset(stream);
        }
    }
}
