use std::sync::{Arc, Mutex};

use super::command::Runner;
use super::native::Native;
use crate::domain::{Action, Application, Manager, Source};

#[derive(Debug, Default)]
struct Stub {
    calls: Mutex<Vec<Vec<String>>>,
}

impl Runner for Stub {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let mut call = vec![program.to_owned()];
        call.extend(args.iter().map(|arg| (*arg).to_owned()));
        self.calls.lock().unwrap().push(call);
        match (program, args.first().copied()) {
            ("pacman", Some("-Ss")) => Ok("extra/gimp 3.0-1\n    Image editor\n".into()),
            ("pacman", Some("-Qq")) => Ok("gimp\n".into()),
            ("pacman", Some("-Qqe")) => Ok("gimp\n".into()),
            ("apt-cache", Some("search")) => Ok("gimp - Image editor\n".into()),
            ("dpkg-query", _) => Ok("gimp\tinstall ok installed\n".into()),
            ("apt-mark", _) => Ok("gimp\n".into()),
            ("dnf", Some("repoquery")) if args.contains(&"--userinstalled") => Ok("gimp\n".into()),
            ("dnf", Some("repoquery")) => Ok("gimp\tImage editor\t3.0\n".into()),
            ("rpm", Some("-qa")) => Ok("gimp\n".into()),
            ("pkexec", _) => Ok(String::new()),
            _ => Err("unexpected command".into()),
        }
    }
}

#[test]
fn apt_e_dnf_buscam_e_operam_com_seus_gerenciadores() {
    for (release, manager, executable) in [
        ("ID=ubuntu\n", Manager::Apt, "apt-get"),
        ("ID=fedora\n", Manager::Dnf, "dnf"),
    ] {
        let runner = Arc::new(Stub::default());
        let native = Native::from_release(runner.clone(), release);
        let found = native.search("gimp").unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].installed);
        assert_eq!(native.installed().unwrap()[0].id, "gimp");
        native.perform(&found[0], Action::Install, manager).unwrap();
        native.perform(&found[0], Action::Update, manager).unwrap();
        native.perform(&found[0], Action::Remove, manager).unwrap();
        let calls = runner.calls.lock().unwrap();
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.first() == Some(&"pkexec".to_owned()))
                .count(),
            3
        );
        assert!(
            calls
                .iter()
                .any(|call| call.get(1).is_some_and(|program| program == executable))
        );
    }
}

#[test]
fn busca_e_lista_pacotes_instalados_no_pacman() {
    let native = Native::from_release(Arc::new(Stub::default()), "ID=arch\n");
    let found = native.search("gimp").unwrap();
    assert_eq!(found.len(), 1);
    assert!(found[0].installed);
    assert_eq!(native.installed().unwrap()[0].id, "gimp");
}

#[test]
fn atualizacao_arch_e_do_sistema_inteiro_e_id_invalido_e_rejeitado() {
    let runner = Arc::new(Stub::default());
    let native = Native::from_release(runner.clone(), "ID=arch\n");
    let mut app = Application {
        id: "gimp".into(),
        name: "gimp".into(),
        summary: String::new(),
        version: None,
        source: Source::Native(Manager::Pacman),
        installed: true,
    };
    native
        .perform(&app, Action::Update, Manager::Pacman)
        .unwrap();
    assert!(
        runner
            .calls
            .lock()
            .unwrap()
            .iter()
            .any(|call| call == &["pkexec", "pacman", "-Syu", "--noconfirm"])
    );
    app.id = "--root".into();
    assert!(
        native
            .perform(&app, Action::Remove, Manager::Pacman)
            .is_err()
    );
}
