use std::io;

pub fn ask_to_unlock(pid: i32) -> io::Result<()> {
    if pid <= 0 {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    match unsafe { libc::kill(pid, libc::SIGUSR1) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::os::unix::process::ExitStatusExt;
    use std::process::Command;

    use super::ask_to_unlock;

    #[test]
    fn o_bloqueador_recebe_o_sigusr1() -> io::Result<()> {
        let mut child = Command::new("sleep").arg("30").spawn()?;
        let pid =
            i32::try_from(child.id()).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        ask_to_unlock(pid)?;
        assert_eq!(child.wait()?.signal(), Some(libc::SIGUSR1));
        Ok(())
    }

    #[test]
    fn pid_zero_ou_negativo_nunca_vira_sinal_para_um_grupo() {
        for pid in [0, -1, -42] {
            assert_eq!(
                ask_to_unlock(pid).map_err(|error| error.kind()),
                Err(io::ErrorKind::InvalidInput)
            );
        }
    }
}
