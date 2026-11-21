pub const GAME_LAUNCHERS: [&str; 2] = ["reaper", "pressure-vessel"];
pub const GAME_HOSTS: [&str; 2] = ["gamescope", "gamescope-wl"];
pub const RECORDERS: [&str; 2] = ["obs", "obs-studio"];
pub const CALLS: [&str; 5] = ["discord", "vesktop", "zoom", "teams-for-linux", "mumble"];
pub const COMPILERS: [&str; 8] = [
    "rustc", "cc1", "cc1plus", "clang", "clang++", "ld.lld", "mold", "javac",
];
pub const EDITORS: [&str; 9] = [
    "code",
    "codium",
    "code-oss",
    "nvim",
    "zed",
    "idea",
    "rust-analyzer",
    "clangd",
    "hx",
];

pub fn listed(list: &[&str], name: &str) -> bool {
    list.iter().any(|entry| name.eq_ignore_ascii_case(entry))
}
