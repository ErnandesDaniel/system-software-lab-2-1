pub fn is_temp(name: &str) -> bool {
    name.len() >= 2 && name.starts_with('t') && name[1..].chars().all(|c| c.is_ascii_digit())
}
