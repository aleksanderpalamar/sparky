use std::io;

const WHO_PROCESS: libc::c_int = 1;
const CLASS_SHIFT: libc::c_int = 13;
const DATA_MASK: libc::c_int = 0xff;

pub fn io_priority(task: u32) -> io::Result<(u8, u8)> {
    let value = unsafe { libc::syscall(libc::SYS_ioprio_get, WHO_PROCESS, task as libc::c_int) };
    if value < 0 {
        return Err(io::Error::last_os_error());
    }
    let value = value as libc::c_int;
    Ok(((value >> CLASS_SHIFT) as u8, (value & DATA_MASK) as u8))
}

pub fn set_io_priority(task: u32, class: u8, data: u8) -> io::Result<()> {
    let value = (libc::c_int::from(class) << CLASS_SHIFT) | libc::c_int::from(data);
    let result = unsafe {
        libc::syscall(
            libc::SYS_ioprio_set,
            WHO_PROCESS,
            task as libc::c_int,
            value,
        )
    };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn nice(task: u32) -> io::Result<i8> {
    unsafe { *libc::__errno_location() = 0 };
    let value = unsafe { libc::getpriority(libc::PRIO_PROCESS, task) };
    let error = io::Error::last_os_error();
    if value == -1 && error.raw_os_error().is_some_and(|code| code != 0) {
        return Err(error);
    }
    Ok(value as i8)
}

pub fn set_nice(task: u32, value: i8) -> io::Result<()> {
    let result = unsafe { libc::setpriority(libc::PRIO_PROCESS, task, libc::c_int::from(value)) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn affinity(task: u32) -> io::Result<Vec<u32>> {
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::cpu_set_t>();
    let result = unsafe { libc::sched_getaffinity(task as libc::pid_t, size, &mut set) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    let limit = (size * 8) as u32;
    Ok((0..limit)
        .filter(|cpu| unsafe { libc::CPU_ISSET(*cpu as usize, &set) })
        .collect())
}

pub fn set_affinity(task: u32, cpus: &[u32]) -> io::Result<()> {
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    for cpu in cpus {
        unsafe { libc::CPU_SET(*cpu as usize, &mut set) };
    }
    let size = std::mem::size_of::<libc::cpu_set_t>();
    let result = unsafe { libc::sched_setaffinity(task as libc::pid_t, size, &set) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn clock_ticks() -> f64 {
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks <= 0 { 100.0 } else { ticks as f64 }
}
