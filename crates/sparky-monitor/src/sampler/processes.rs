use crate::domain::gpu::{GpuUse, combine, parse_fdinfo};
use crate::domain::process::{
    display_name, io_bytes, kernel_thread, parse_stat, program, real_uid, resident_pages,
};
use crate::domain::snapshot::ProcessSample;
use crate::ports::SystemFiles;

use super::digits;

pub fn processes(files: &dyn SystemFiles, page_size: u64) -> Vec<ProcessSample> {
    files
        .list("/proc")
        .into_iter()
        .filter(|name| digits(name))
        .filter_map(|pid| process(files, &format!("/proc/{pid}"), page_size))
        .collect()
}

fn process(files: &dyn SystemFiles, base: &str, page_size: u64) -> Option<ProcessSample> {
    let stat = parse_stat(&files.read(&format!("{base}/stat"))?)?;
    if kernel_thread(&stat) {
        return None;
    }
    let cmdline = files.read(&format!("{base}/cmdline")).unwrap_or_default();
    let pages = files
        .read(&format!("{base}/statm"))
        .and_then(|text| resident_pages(&text))
        .unwrap_or_default();
    Some(ProcessSample {
        name: display_name(&stat.comm, &cmdline),
        program: program(&stat.comm, &cmdline),
        user: files
            .read(&format!("{base}/status"))
            .and_then(|text| real_uid(&text)),
        memory: pages * page_size,
        io: files
            .read(&format!("{base}/io"))
            .and_then(|text| io_bytes(&text)),
        gpu: gpu_use(files, base),
        stat,
    })
}

fn gpu_use(files: &dyn SystemFiles, base: &str) -> Option<GpuUse> {
    let clients = files
        .list(&format!("{base}/fd"))
        .into_iter()
        .filter(|fd| {
            files
                .link(&format!("{base}/fd/{fd}"))
                .is_some_and(|target| target.starts_with("/dev/dri/"))
        })
        .filter_map(|fd| {
            files
                .read(&format!("{base}/fdinfo/{fd}"))
                .and_then(|text| parse_fdinfo(&text))
        });
    combine(clients)
}
