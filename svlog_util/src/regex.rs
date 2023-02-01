use regex::{Regex, RegexBuilder};

lazy_static! {
    pub static ref RE_DATE: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    pub static ref RE_TIME_MIN: Regex = Regex::new(r"^\d{2}:\d{2}$").unwrap();
    pub static ref RE_TIME_SEC: Regex = Regex::new(r"^\d{2}:\d{2}:\d{2}$").unwrap();
    pub static ref RE_DATETIME_MIN: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$").unwrap();
    pub static ref RE_DATETIME_SEC: Regex =
        Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
}

pub fn build_regex(pattern: &Option<String>, case_insensitive: bool) -> Option<Regex> {
    pattern.as_ref().map(|pattern| {
        RegexBuilder::new(&pattern[..])
            .case_insensitive(case_insensitive)
            .build()
            .unwrap()
    })
}
