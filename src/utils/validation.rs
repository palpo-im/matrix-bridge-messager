use regex::Regex;

pub fn validate_phone_number(phone_number: &str) -> bool {
    static PHONE_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"^\+?[0-9][0-9\-\s]{5,30}$").expect("phone regex must be valid")
    });
    PHONE_RE.is_match(phone_number.trim())
}

pub fn sanitize_message_text(input: &str, max_len: usize) -> String {
    let normalized = input
        .replace('\0', "")
        .replace('\r', "")
        .trim()
        .to_string();

    if normalized.chars().count() <= max_len {
        normalized
    } else {
        normalized.chars().take(max_len).collect()
    }
}

pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::{escape_html, sanitize_message_text, validate_phone_number};

    #[test]
    fn validate_phone_numbers() {
        assert!(validate_phone_number("+1234567890"));
        assert!(validate_phone_number("123 456 7890"));
        assert!(!validate_phone_number("abc"));
        assert!(!validate_phone_number("12"));
    }

    #[test]
    fn sanitize_message_text_trims_and_limits() {
        assert_eq!(sanitize_message_text(" hello \r\n", 100), "hello");
        assert_eq!(sanitize_message_text("abcdef", 3), "abc");
    }

    #[test]
    fn escape_html_works() {
        assert_eq!(
            escape_html("<b>\"quote\" & test</b>"),
            "&lt;b&gt;&quot;quote&quot; &amp; test&lt;/b&gt;"
        );
    }
}
