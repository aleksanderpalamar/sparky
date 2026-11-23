const ESCAPED: [char; 4] = ['"', '`', '$', '\\'];
const ICON_FLAG: &str = "--icon";

pub struct Fields<'a> {
    pub name: &'a str,
    pub icon: Option<&'a str>,
    pub location: &'a str,
}

pub fn arguments(exec: &str, fields: &Fields<'_>) -> Option<Vec<String>> {
    let words = split(exec)?;
    let arguments: Vec<String> = words
        .into_iter()
        .flat_map(|word| expand(word, fields))
        .collect();
    match arguments.first() {
        Some(program) if !program.is_empty() => Some(arguments),
        _ => None,
    }
}

fn split(exec: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut chars = exec.chars();
    while let Some(next) = chars.next() {
        match next {
            '"' => {
                started = true;
                quoted(&mut chars, &mut word)?;
            }
            ' ' | '\t' => {
                if std::mem::take(&mut started) {
                    words.push(std::mem::take(&mut word));
                }
            }
            other => {
                started = true;
                word.push(other);
            }
        }
    }
    if started {
        words.push(word);
    }
    Some(words)
}

fn quoted(chars: &mut std::str::Chars<'_>, word: &mut String) -> Option<()> {
    while let Some(next) = chars.next() {
        match next {
            '"' => return Some(()),
            '\\' => {
                let escaped = chars.next()?;
                if !ESCAPED.contains(&escaped) {
                    word.push('\\');
                }
                word.push(escaped);
            }
            other => word.push(other),
        }
    }
    None
}

fn expand(word: String, fields: &Fields<'_>) -> Vec<String> {
    if word == "%i" {
        return fields
            .icon
            .map(|icon| vec![ICON_FLAG.to_owned(), icon.to_owned()])
            .unwrap_or_default();
    }
    let mut expanded = String::with_capacity(word.len());
    let mut chars = word.chars().peekable();
    let mut dropped = false;
    while let Some(next) = chars.next() {
        match (next, chars.peek().copied()) {
            ('%', Some(code)) => {
                chars.next();
                match code {
                    '%' => expanded.push('%'),
                    'c' => expanded.push_str(fields.name),
                    'k' => expanded.push_str(fields.location),
                    _ => dropped = true,
                }
            }
            (other, _) => expanded.push(other),
        }
    }
    match dropped && expanded.is_empty() {
        true => Vec::new(),
        false => vec![expanded],
    }
}
