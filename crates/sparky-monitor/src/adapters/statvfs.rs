use crate::ports::Filesystems;

pub struct Statvfs;

impl Filesystems for Statvfs {
    fn usage(&self, mount: &str) -> Option<(u64, u64)> {
        let stats = rustix::fs::statvfs(mount).ok()?;
        let total = stats.f_blocks * stats.f_frsize;
        let free = stats.f_bfree * stats.f_frsize;
        Some((total, total.saturating_sub(free)))
    }
}
