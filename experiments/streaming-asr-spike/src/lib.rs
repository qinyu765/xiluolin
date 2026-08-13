use std::collections::HashSet;

const CHUNK_MILLIS: usize = 100;
const TAIL_PADDING_MILLIS: usize = 300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub encoder: String,
    pub decoder: String,
    pub joiner: String,
    pub tokens: String,
    pub wav: String,
    pub hotwords: Vec<String>,
    pub realtime: bool,
    pub repeat: usize,
}

impl CliArgs {
    pub fn parse_from<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut encoder = None;
        let mut decoder = None;
        let mut joiner = None;
        let mut tokens = None;
        let mut wav = None;
        let mut hotwords = Vec::new();
        let mut realtime = false;
        let mut repeat = 1;
        let mut args = args.into_iter().skip(1);

        while let Some(argument) = args.next() {
            let argument = argument.as_ref();
            match argument {
                "--encoder" => encoder = Some(next_value(&mut args, argument)?),
                "--decoder" => decoder = Some(next_value(&mut args, argument)?),
                "--joiner" => joiner = Some(next_value(&mut args, argument)?),
                "--tokens" => tokens = Some(next_value(&mut args, argument)?),
                "--wav" => wav = Some(next_value(&mut args, argument)?),
                "--hotword" => hotwords.push(next_value(&mut args, argument)?),
                "--realtime" => realtime = true,
                "--repeat" => {
                    let value = next_value(&mut args, argument)?;
                    repeat = value
                        .parse::<usize>()
                        .map_err(|_| "--repeat 必须是正整数".to_string())?;
                    if repeat == 0 {
                        return Err("--repeat 必须大于 0".to_string());
                    }
                }
                unknown => return Err(format!("未知参数：{unknown}")),
            }
        }

        Ok(Self {
            encoder: required(encoder, "--encoder")?,
            decoder: required(decoder, "--decoder")?,
            joiner: required(joiner, "--joiner")?,
            tokens: required(tokens, "--tokens")?,
            wav: required(wav, "--wav")?,
            hotwords: normalize_hotwords(&hotwords),
            realtime,
            repeat,
        })
    }
}

fn next_value<I, S>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    args.next()
        .map(|value| value.as_ref().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{flag} 缺少参数值"))
}

fn required(value: Option<String>, flag: &str) -> Result<String, String> {
    value.ok_or_else(|| format!("缺少必填参数 {flag}"))
}

pub fn normalize_hotwords(hotwords: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for hotword in hotwords {
        let hotword = hotword.trim();
        if !hotword.is_empty() && seen.insert(hotword.to_string()) {
            normalized.push(hotword.to_string());
        }
    }
    normalized
}

pub fn chunk_ranges(sample_count: usize, sample_rate: u32) -> Vec<(usize, usize)> {
    if sample_rate == 0 {
        return Vec::new();
    }
    let chunk_size = (sample_rate as usize * CHUNK_MILLIS / 1_000).max(1);
    (0..sample_count)
        .step_by(chunk_size)
        .map(|start| (start, (start + chunk_size).min(sample_count)))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedAction {
    Audio { start: usize, end: usize },
    TailPadding { samples: usize },
    InputFinished,
}

pub fn build_feed_plan(sample_count: usize, sample_rate: u32) -> Vec<FeedAction> {
    if sample_rate == 0 {
        return vec![FeedAction::InputFinished];
    }
    let mut actions = chunk_ranges(sample_count, sample_rate)
        .into_iter()
        .map(|(start, end)| FeedAction::Audio { start, end })
        .collect::<Vec<_>>();
    actions.push(FeedAction::TailPadding {
        samples: sample_rate as usize * TAIL_PADDING_MILLIS / 1_000,
    });
    actions.push(FeedAction::InputFinished);
    actions
}

#[derive(Debug, Default)]
pub struct RevisionTracker {
    revision: u64,
    last_text: String,
}

impl RevisionTracker {
    pub fn observe(&mut self, text: &str) -> Option<u64> {
        let text = text.trim();
        if text.is_empty() || text == self.last_text {
            return None;
        }
        self.revision += 1;
        self.last_text = text.to_string();
        Some(self.revision)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_feed_plan, chunk_ranges, normalize_hotwords, CliArgs, FeedAction, RevisionTracker,
    };

    #[test]
    fn chunk_ranges_cover_every_sample_once() {
        assert_eq!(
            chunk_ranges(3_250, 16_000),
            vec![(0, 1_600), (1_600, 3_200), (3_200, 3_250)]
        );
    }

    #[test]
    fn chunk_ranges_reject_zero_sample_rate() {
        assert_eq!(chunk_ranges(100, 0), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn hotwords_are_trimmed_filtered_and_stably_deduplicated() {
        assert_eq!(
            normalize_hotwords(&[
                " Next.js ".to_string(),
                "".to_string(),
                "XiLuoLin".to_string(),
                "Next.js".to_string(),
                "   ".to_string(),
            ]),
            vec!["Next.js".to_string(), "XiLuoLin".to_string()]
        );
    }

    #[test]
    fn revision_only_advances_for_new_nonempty_hypotheses() {
        let mut tracker = RevisionTracker::default();
        assert_eq!(tracker.observe(""), None);
        assert_eq!(tracker.observe("你好"), Some(1));
        assert_eq!(tracker.observe("你好"), None);
        assert_eq!(tracker.observe("你好世界"), Some(2));
    }

    #[test]
    fn feed_plan_appends_tail_padding_before_finishing_input() {
        assert_eq!(
            build_feed_plan(1_600, 16_000),
            vec![
                FeedAction::Audio {
                    start: 0,
                    end: 1_600
                },
                FeedAction::TailPadding { samples: 4_800 },
                FeedAction::InputFinished,
            ]
        );
    }

    #[test]
    fn cli_parser_accepts_repeated_hotwords_and_repeat_count() {
        let args = CliArgs::parse_from([
            "spike",
            "--encoder",
            "encoder.onnx",
            "--decoder",
            "decoder.onnx",
            "--joiner",
            "joiner.onnx",
            "--tokens",
            "tokens.txt",
            "--wav",
            "sample.wav",
            "--hotword",
            "Next.js",
            "--hotword",
            "XiLuoLin",
            "--realtime",
            "--repeat",
            "10",
        ])
        .expect("valid arguments should parse");

        assert_eq!(args.hotwords, vec!["Next.js", "XiLuoLin"]);
        assert!(args.realtime);
        assert_eq!(args.repeat, 10);
    }

    #[test]
    fn cli_parser_rejects_missing_required_paths() {
        let error = CliArgs::parse_from(["spike", "--repeat", "2"])
            .expect_err("missing model paths must fail");
        assert!(error.contains("--encoder"));
    }
}
