use super::models::Hotword;

pub(super) fn format_hotword_context(hotwords: &[Hotword]) -> String {
    hotwords
        .iter()
        .map(|hotword| {
            if hotword.category.trim().is_empty() {
                format!("- {}", hotword.text)
            } else {
                format!("- {}（{}）", hotword.text, hotword.category)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn bool_to_int(value: bool) -> i64 {
    i64::from(value)
}

pub(super) fn int_to_bool(value: i64) -> bool {
    value != 0
}
