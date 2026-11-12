use std::sync::{Arc, Mutex};

use super::command::{CommandOutput, Runner};
use super::native::Native;
use crate::domain::Manager;

#[derive(Debug, Default)]
struct Stub {
    calls: Mutex<Vec<Vec<String>>>,
}

impl Runner for Stub {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let mut call = vec![program.to_owned()];
        call.extend(args.iter().map(|arg| (*arg).to_owned()));
        self.calls.lock().unwrap().push(call);
        match (program, args) {
            ("apt-get", ["-s", "--with-new-pkgs", "upgrade"]) => {
                Ok("Inst libc6:amd64 [2.36] (2.37 Debian:stable [amd64])\n".into())
            }
            ("rpm", _) => Ok("mesa.x86_64\t24.1-1\n".into()),
            ("pkexec", _) => Ok(String::new()),
            _ => Err("unexpected command".into()),
        }
    }

    fn run_status(&self, program: &str, args: &[&str]) -> Result<CommandOutput, String> {
        let mut call = vec![program.to_owned()];
        call.extend(args.iter().map(|arg| (*arg).to_owned()));
        self.calls.lock().unwrap().push(call);
        match program {
            "checkupdates" => Ok(CommandOutput {
                stdout: "mesa 24.1-1 -> 24.2-1\n".into(),
                stderr: String::new(),
                code: Some(0),
            }),
            "dnf" => Ok(CommandOutput {
                stdout: "mesa.x86_64 24.2-1 updates\n".into(),
                stderr: String::new(),
                code: Some(100),
            }),
            _ => Err("unexpected command".into()),
        }
    }
}

#[test]
fn consulta_arch_e_aplica_uma_transacao_integral() {
    let runner = Arc::new(Stub::default());
    let native = Native::from_release(runner.clone(), "ID=arch\n");
    let updates = native.updates(false).unwrap();
    assert_eq!(updates[0].id, "mesa");
    native.apply_update(Manager::Pacman, Some("mesa")).unwrap();
    let calls = runner.calls.lock().unwrap();
    assert!(
        calls.contains(
            &vec!["pkexec", "pacman", "-Syu", "--noconfirm"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        )
    );
}

#[test]
fn apt_recarrega_indices_somente_ao_pedir_e_atualiza_sistema() {
    let runner = Arc::new(Stub::default());
    let native = Native::from_release(runner.clone(), "ID=debian\n");
    assert_eq!(native.updates(false).unwrap()[0].id, "libc6:amd64");
    assert_eq!(runner.calls.lock().unwrap().len(), 1);
    assert_eq!(native.updates(true).unwrap().len(), 1);
    native.apply_update(Manager::Apt, None).unwrap();
    let calls = runner.calls.lock().unwrap();
    assert!(
        calls.contains(
            &vec!["pkexec", "apt-get", "update"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        )
    );
    assert!(
        calls.contains(
            &vec!["pkexec", "apt-get", "upgrade", "--with-new-pkgs", "-y"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        )
    );
}

#[test]
fn dnf_interpreta_codigo_cem_e_versoes_instaladas() {
    let native = Native::from_release(Arc::new(Stub::default()), "ID=fedora\n");
    let updates = native.updates(false).unwrap();
    assert_eq!(updates[0].installed.as_deref(), Some("24.1-1"));
    assert_eq!(updates[0].available.as_deref(), Some("24.2-1"));
}

#[derive(Debug)]
struct StatusStub {
    code: i32,
    message: &'static str,
}

impl Runner for StatusStub {
    fn run(&self, program: &str, _: &[&str]) -> Result<String, String> {
        if program == "rpm" {
            return Ok("mesa.x86_64\t24.1-1\n".into());
        }
        Err("unexpected command".into())
    }

    fn run_status(&self, _: &str, _: &[&str]) -> Result<CommandOutput, String> {
        Ok(CommandOutput {
            stdout: String::new(),
            stderr: self.message.into(),
            code: Some(self.code),
        })
    }
}

#[test]
fn codigos_sem_atualizacoes_nao_viram_erro() {
    let arch = Native::from_release(
        Arc::new(StatusStub {
            code: 2,
            message: "",
        }),
        "ID=arch\n",
    );
    let fedora = Native::from_release(
        Arc::new(StatusStub {
            code: 0,
            message: "",
        }),
        "ID=fedora\n",
    );
    assert!(arch.updates(false).unwrap().is_empty());
    assert!(fedora.updates(false).unwrap().is_empty());
}

#[test]
fn falhas_de_consulta_nao_parecem_sistema_atualizado() {
    let arch = Native::from_release(
        Arc::new(StatusStub {
            code: 1,
            message: "network failed",
        }),
        "ID=arch\n",
    );
    let fedora = Native::from_release(
        Arc::new(StatusStub {
            code: 1,
            message: "repo failed",
        }),
        "ID=fedora\n",
    );
    assert_eq!(arch.updates(false).unwrap_err(), "network failed");
    assert_eq!(fedora.updates(false).unwrap_err(), "repo failed");
}
