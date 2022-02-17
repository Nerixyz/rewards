use regex::Regex;

pub fn trim_to(mut text: String, max_length: usize) -> String {
    if text.len() > max_length - 3 {
        text.truncate(max_length - 3);
        text.push_str("...");
    }
    text
}

pub fn first_capture<'a>(text: &'a str, regex: &Regex) -> Option<&'a str> {
    regex
        .captures(text)
        .and_then(|c| c.iter().nth(1).flatten().map(|m| m.as_str()))
}
