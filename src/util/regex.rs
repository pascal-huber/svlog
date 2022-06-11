use regex::{Regex, RegexBuilder};

pub fn build_regex(pattern: &Option<String>) -> Option<Regex> {
    pattern.as_ref().map(|pattern| {
        RegexBuilder::new(&pattern[..])
            .case_insensitive(true)
            .build()
            .unwrap()
    })
}
