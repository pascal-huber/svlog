#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct LogPriority(u8);

impl LogPriority {
    pub fn from_str(s: &str) -> Option<Self> {
        Self::priority_value(s)
    }

    pub fn from_str_or_max(s: &str) -> Self {
        let priority = Self::priority_value(s);
        if let Some(priority) = priority {
            priority
        } else {
            Self::max()
        }
    }

    pub fn min() -> Self {
        LogPriority(0)
    }

    pub fn max() -> Self {
        LogPriority(7)
    }

    fn priority_value(s: &str) -> Option<LogPriority> {
        match s {
            "0" | "emerg" => Some(LogPriority(0)),
            "1" | "alert" => Some(LogPriority(1)),
            "2" | "crit" => Some(LogPriority(2)),
            "3" | "err" => Some(LogPriority(3)),
            "4" | "warn" => Some(LogPriority(4)),
            "5" | "notice" => Some(LogPriority(5)),
            "6" | "info" => Some(LogPriority(6)),
            "7" | "debug" => Some(LogPriority(7)),
            _ => None,
        }
    }
}
