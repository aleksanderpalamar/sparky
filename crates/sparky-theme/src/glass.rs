pub fn glass_in(contents: &str) -> bool {
    sparky_config::parse(contents).is_ok_and(|config| config.general.frosted_glass)
}

pub fn glass_from_env() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = sparky_config::path(std::env::var("XDG_CONFIG_HOME").ok().as_deref(), &home);
    std::fs::read_to_string(path).is_ok_and(|contents| glass_in(&contents))
}
