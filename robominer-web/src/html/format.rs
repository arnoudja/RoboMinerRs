use std::fmt;

pub(crate) fn selected_attr(selected: bool) -> &'static str {
    if selected { " selected" } else { "" }
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// HTML text/attribute content that has already been escaped.
///
/// Construct with [`EscapedHtml::from_untrusted`] (or `From<&str>`) so callers
/// cannot accidentally embed raw user/robot/ore names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EscapedHtml(String);

impl EscapedHtml {
    pub(crate) fn from_untrusted(value: &str) -> Self {
        Self(escape_html(value))
    }
}

impl fmt::Display for EscapedHtml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for EscapedHtml {
    fn from(value: &str) -> Self {
        Self::from_untrusted(value)
    }
}

impl From<String> for EscapedHtml {
    fn from(value: String) -> Self {
        Self::from_untrusted(&value)
    }
}

pub(crate) fn format_utc_millis(millis: i64) -> String {
    let seconds = millis.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

pub(crate) fn format_relative_time_millis(event_millis: i64, now_millis: i64) -> String {
    let diff_seconds = (now_millis - event_millis).div_euclid(1000);
    if diff_seconds < 0 {
        return format_utc_millis(event_millis);
    }
    if diff_seconds < 45 {
        return "just now".to_string();
    }
    if diff_seconds < 90 {
        return "1 minute ago".to_string();
    }
    let minutes = diff_seconds / 60;
    if minutes < 45 {
        return format!("{minutes} minutes ago");
    }
    if minutes < 90 {
        return "1 hour ago".to_string();
    }
    let hours = minutes / 60;
    if hours < 24 {
        return if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{hours} hours ago")
        };
    }
    if hours < 48 {
        return "1 day ago".to_string();
    }
    let days = hours / 24;
    if days < 7 {
        return if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{days} days ago")
        };
    }
    if days < 14 {
        return "1 week ago".to_string();
    }
    let weeks = days / 7;
    if weeks < 5 {
        return if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{weeks} weeks ago")
        };
    }

    format_utc_millis(event_millis)
}

pub(crate) fn format_period(seconds: i32) -> String {
    if seconds % 3600 == 0 && seconds > 3600 {
        format!("{} hours", seconds / 3600)
    } else if seconds % 60 == 0 && seconds > 60 {
        format!("{} minutes", seconds / 60)
    } else {
        format!("{seconds} seconds")
    }
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        .div_euclid(365);
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2).div_euclid(153);
    let day = day_of_year - (153 * month_prime + 2).div_euclid(5) + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escaped_html_escapes_untrusted_input_on_display() {
        let escaped = EscapedHtml::from_untrusted(r#"A <b> "&'"#);
        assert_eq!(escaped.to_string(), "A &lt;b&gt; &quot;&amp;&#39;");
    }
}
