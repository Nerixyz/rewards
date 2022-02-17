pub fn trim_to(mut text: String, max_length: usize) -> String {
    if text.len() > max_length - 3 {
        text.truncate(max_length - 3);
        text.push_str("...");
    }
    text
}
