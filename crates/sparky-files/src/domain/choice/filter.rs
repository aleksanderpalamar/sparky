#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    Glob(String),
    Mime(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub name: String,
    pub rules: Vec<Rule>,
}

impl Filter {
    pub fn accepts(&self, name: &str, mime_of: impl Fn(&str) -> String) -> bool {
        self.rules.iter().any(|rule| match rule {
            Rule::Glob(pattern) => glob(&lower(pattern), &lower(name)),
            Rule::Mime(wanted) => mime_matches(wanted, &mime_of(name)),
        })
    }
}

fn lower(text: &str) -> Vec<char> {
    text.to_lowercase().chars().collect()
}

fn glob(pattern: &[char], text: &[char]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some(('*', rest)) => (0..=text.len()).any(|skip| glob(rest, &text[skip..])),
        Some(('?', rest)) => !text.is_empty() && glob(rest, &text[1..]),
        Some(('[', rest)) => class(rest, text),
        Some((wanted, rest)) => text.first() == Some(wanted) && glob(rest, &text[1..]),
    }
}

fn class(rest: &[char], text: &[char]) -> bool {
    let Some(end) = rest.iter().position(|letter| *letter == ']') else {
        return text.first() == Some(&'[') && glob(rest, &text[1..]);
    };
    let Some(first) = text.first() else {
        return false;
    };
    rest[..end].contains(first) && glob(&rest[end + 1..], &text[1..])
}

fn mime_matches(wanted: &str, actual: &str) -> bool {
    let wanted = wanted.trim().to_ascii_lowercase();
    let actual = actual.to_ascii_lowercase();
    match wanted.strip_suffix("/*") {
        Some("*") => true,
        Some(family) => actual.split('/').next() == Some(family),
        None => wanted == actual,
    }
}
