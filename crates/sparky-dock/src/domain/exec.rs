const ESCAPED: [char; 4] = ['"', '`', '$', '\\'];

pub fn arguments(exec: &str) -> Option<Vec<String>> {
    let words = split(exec)?;
    let arguments: Vec<String> = words.into_iter().filter_map(expand).collect();
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

fn expand(word: String) -> Option<String> {
    let mut expanded = String::with_capacity(word.len());
    let mut chars = word.chars().peekable();
    let mut codes = 0;
    while let Some(next) = chars.next() {
        match (next, chars.peek().copied()) {
            ('%', Some('%')) => {
                chars.next();
                expanded.push('%');
            }
            ('%', Some(_)) => {
                chars.next();
                codes += 1;
            }
            (other, _) => expanded.push(other),
        }
    }
    match (codes, expanded.is_empty()) {
        (0, _) | (_, false) => Some(expanded),
        _ => None,
    }
}
