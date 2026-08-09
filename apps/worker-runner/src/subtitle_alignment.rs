#[derive(serde::Deserialize)]
pub(crate) struct RawAlignmentFile {
    pub(crate) words: Vec<RawWord>,
}

#[derive(serde::Deserialize)]
pub(crate) struct RawWord {
    pub(crate) word: String,
    pub(crate) start: f64,
    pub(crate) end: f64,
}

pub(crate) fn raw_words_to_provider(words: &[RawWord]) -> Vec<dubbridge_providers::WordAlignment> {
    words
        .iter()
        .map(|w| dubbridge_providers::WordAlignment {
            word: w.word.clone(),
            start_ms: (w.start * 1000.0).round() as u64,
            end_ms: (w.end * 1000.0).round() as u64,
        })
        .collect()
}
