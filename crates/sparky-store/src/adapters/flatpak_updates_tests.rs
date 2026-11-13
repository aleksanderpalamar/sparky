use std::sync::{Arc, Mutex};

use super::command::Runner;
use super::flatpak::Flatpak;
use crate::domain::Installation;
use crate::domain::updates::UpdateKind;

#[derive(Debug, Default)]
struct Stub {
    calls: Mutex<Vec<Vec<String>>>,
}

impl Runner for Stub {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
        assert_eq!(program, "flatpak");
        self.calls
            .lock()
            .unwrap()
            .push(args.iter().map(|arg| (*arg).to_owned()).collect());
        if args[1] == "update" {
            return Ok(String::new());
        }
        assert!(args.contains(&"--all"));
        match (args[0], args[1], args[2]) {
            ("--user", "list", "--app") => Ok("org.gimp.GIMP/x86_64/beta\t3.0\tflathub\n".into()),
            ("--system", "list", "--runtime") => Ok("org.freedesktop.Platform/x86_64/24.08\t24.08\tflathub\norg.freedesktop.Platform.Locale/x86_64/24.08\t\tflathub\n".into()),
            ("--user", "remote-ls", "--app") => Ok("app/org.gimp.GIMP/x86_64/beta\tGIMP\t3.1\tflathub\napp/org.other.App/x86_64/stable\tOther\t1\tother\n".into()),
            ("--system", "remote-ls", "--runtime") => Ok("runtime/org.freedesktop.Platform/x86_64/24.08\tPlatform\t24.08.1\tflathub\nruntime/org.freedesktop.Platform.Locale/x86_64/24.08\tLocale\t\tflathub\n".into()),
            (_, "list" | "remote-ls", _) => Ok(String::new()),
            _ => Err("unexpected command".into()),
        }
    }
}

#[test]
fn consulta_ref_completo_por_escopo_e_ignora_remota_nao_instalada() {
    let flatpak = Flatpak::new(Arc::new(Stub::default()));
    let user = flatpak.updates(Installation::User).unwrap();
    let system = flatpak.updates(Installation::System).unwrap();
    assert_eq!(user.len(), 1);
    assert_eq!(user[0].id, "app/org.gimp.GIMP/x86_64/beta");
    assert_eq!(user[0].installed.as_deref(), Some("3.0"));
    assert_eq!(system[0].kind, UpdateKind::Runtime);
    assert_eq!(
        system[1].id,
        "runtime/org.freedesktop.Platform.Locale/x86_64/24.08"
    );
}

#[test]
fn atualiza_item_exato_ou_grupo_sem_confundir_escopos() {
    let runner = Arc::new(Stub::default());
    let flatpak = Flatpak::new(runner.clone());
    flatpak
        .apply_update(Installation::User, Some("app/org.gimp.GIMP/x86_64/beta"))
        .unwrap();
    flatpak.apply_update(Installation::System, None).unwrap();
    assert!(
        flatpak
            .apply_update(Installation::User, Some("--system"))
            .is_err()
    );
    let calls = runner.calls.lock().unwrap();
    assert!(
        calls.contains(
            &vec![
                "--user",
                "update",
                "-y",
                "--noninteractive",
                "--",
                "app/org.gimp.GIMP/x86_64/beta"
            ]
            .into_iter()
            .map(str::to_owned)
            .collect()
        )
    );
    assert!(
        calls.contains(
            &vec!["--system", "update", "-y", "--noninteractive"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        )
    );
    assert_eq!(calls.len(), 2);
}
