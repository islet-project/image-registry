use regex::Regex;

pub const PATTERN: &str = r"^[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}$";

pub fn verify<T>(tag: T) -> bool
where
    T: AsRef<str>,
{
    let re = Regex::new(PATTERN).unwrap();
    re.is_match(tag.as_ref())
}
