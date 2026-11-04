pub fn joined<const N: usize>(parts: [Option<String>; N]) -> String {
    parts.into_iter().flatten().collect::<Vec<_>>().join(" · ")
}

pub fn translated(key: &str, values: &[(&str, String)]) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    for (name, value) in values {
        args.set(*name, value.as_str());
    }
    crate::i18n::message(key, Some(&args))
}
