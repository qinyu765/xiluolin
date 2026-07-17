use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Once, OnceLock},
};

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

static MODEL_CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<WhisperContext>>>> = OnceLock::new();
static LOG_HOOK: Once = Once::new();

fn cache() -> &'static Mutex<HashMap<PathBuf, Arc<WhisperContext>>> {
    MODEL_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn load_context(model_path: &Path) -> Result<Arc<WhisperContext>, String> {
    LOG_HOOK.call_once(whisper_rs::install_logging_hooks);
    if !model_path.exists() {
        return Err("本地 ASR 模型尚未下载".to_string());
    }
    let canonical = std::fs::canonicalize(model_path).map_err(|error| error.to_string())?;
    if let Some(context) = cache()
        .lock()
        .map_err(|error| error.to_string())?
        .get(&canonical)
        .cloned()
    {
        return Ok(context);
    }
    let context = Arc::new(
        WhisperContext::new_with_params(&canonical, WhisperContextParameters::default())
            .map_err(|error| format!("加载本地 ASR 模型失败：{error}"))?,
    );
    cache()
        .lock()
        .map_err(|error| error.to_string())?
        .insert(canonical, context.clone());
    Ok(context)
}

pub fn clear_model_cache() {
    if let Some(cache) = MODEL_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.clear();
        }
    }
}

pub fn verify_model(model_path: &Path) -> Result<(), String> {
    load_context(model_path).map(|_| ())
}

pub fn transcribe(audio_path: &Path, model_path: &Path) -> Result<String, String> {
    let context = load_context(model_path)?;
    let audio = load_wav_16khz_mono(audio_path)?;
    let mut state = context
        .create_state()
        .map_err(|error| format!("创建本地 ASR 状态失败：{error}"))?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(
        std::thread::available_parallelism()
            .map(|value| value.get().min(8) as i32)
            .unwrap_or(4),
    );
    params.set_translate(false);
    params.set_language(None);
    params.set_no_context(true);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    state
        .full(params, &audio)
        .map_err(|error| format!("本地 ASR 推理失败：{error}"))?;

    let text = state
        .as_iter()
        .map(|segment| segment.to_string())
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();
    if text.is_empty() {
        return Err("本地 ASR 未识别出文本".to_string());
    }
    Ok(text)
}

fn load_wav_16khz_mono(audio_path: &Path) -> Result<Vec<f32>, String> {
    let mut reader = hound::WavReader::open(audio_path)
        .map_err(|error| format!("读取本地 ASR WAV 失败：{error}"))?;
    let spec = reader.spec();
    let channels = spec.channels.max(1) as usize;
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| sample.map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int if spec.bits_per_sample <= 16 => reader
            .samples::<i16>()
            .map(|sample| {
                sample
                    .map(|value| value as f32 / i16::MAX as f32)
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let scale = ((1_i64 << (spec.bits_per_sample - 1)) - 1) as f32;
            reader
                .samples::<i32>()
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / scale)
                        .map_err(|error| error.to_string())
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let mono = samples
        .chunks(channels)
        .map(|frame| frame.iter().copied().sum::<f32>() / frame.len() as f32)
        .collect::<Vec<_>>();
    Ok(resample_linear(&mono, spec.sample_rate, 16_000))
}

fn resample_linear(input: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if input.is_empty() || source_rate == target_rate {
        return input.to_vec();
    }
    let output_len = ((input.len() as u64 * target_rate as u64) / source_rate as u64) as usize;
    let ratio = source_rate as f64 / target_rate as f64;
    (0..output_len)
        .map(|index| {
            let position = index as f64 * ratio;
            let left = position.floor() as usize;
            let right = (left + 1).min(input.len() - 1);
            let fraction = (position - left as f64) as f32;
            input[left] * (1.0 - fraction) + input[right] * fraction
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_resampling_changes_sample_count() {
        let input = vec![0.0; 48_000];
        let output = resample_linear(&input, 48_000, 16_000);
        assert_eq!(output.len(), 16_000);
    }

    #[test]
    fn equal_sample_rate_keeps_audio() {
        let input = vec![0.1, 0.2, 0.3];
        assert_eq!(resample_linear(&input, 16_000, 16_000), input);
    }
}
