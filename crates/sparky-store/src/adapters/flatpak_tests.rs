use std::sync::{Arc, Mutex};

use super::command::Runner;
use super::flatpak::Flatpak;
use crate::domain::{Action, Installation, Source};

#[derive(Debug, Default)]
struct Stub {
    calls: Mutex<Vec<Vec<String>>>,
}

impl Runner for Stub {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
        assert_eq!(program, "flatpak");
        assert!(!args.contains(&"-j"));
        if matches!(
            args.get(1).copied(),
            Some("remotes" | "remote-ls" | "search" | "list")
        ) {
            assert!(args.iter().any(|arg| arg.starts_with("--columns=")));
        }
        self.calls
            .lock()
            .unwrap()
            .push(args.iter().map(|arg| (*arg).to_owned()).collect());
        match (args.first().copied(), args.get(1).copied()) {
            (Some("--system"), Some("remotes")) => Ok("flathub\n".into()),
            (Some("--user"), Some("remotes")) => Ok(String::new()),
            (Some("--system"), Some("list")) => Ok("GIMP\tImage editor\torg.gimp.GIMP\t3.0\tflathub\nOther\tElsewhere\torg.other.App\t1.0\tother\n".into()),
            (Some("--user"), Some("list")) => Ok(String::new()),
            (Some("--system"), Some("search")) => Ok("org.gimp.GIMP\tGIMP\tImage editor\t3.0\tflathub\tstable\norg.other.App\tElsewhere\t\t1.0\tother\tstable\n".into()),
            (Some("--system"), Some("remote-ls")) => Ok("org.gimp.GIMP\tGIMP\tImage editor\t3.0\tstable\n".into()),
            _ => Ok(String::new()),
        }
    }
}

#[test]
fn navega_catalogo_do_flathub() {
    let runner = Arc::new(Stub::default());
    let flatpak = Flatpak::new(runner.clone());
    let apps = flatpak.browse(Installation::System).unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].id, "org.gimp.GIMP");
    assert!(apps[0].installed);
    assert!(
        runner
            .calls
            .lock()
            .unwrap()
            .iter()
            .any(|args| args.contains(&"remote-ls".to_owned()))
    );
}

#[test]
fn busca_somente_flathub_e_reconhece_instalacao() {
    let runner = Arc::new(Stub::default());
    let flatpak = Flatpak::new(runner);
    let scopes = flatpak.scopes();
    assert_eq!(scopes, vec![Installation::System]);
    let found = flatpak.search("gimp", &scopes).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].source, Source::Flathub(Installation::System));
    assert!(found[0].installed);
    assert_eq!(found[0].summary, "Image editor");
}

#[test]
fn operacao_flatpak_usa_escopo_e_id_exatos() {
    let runner = Arc::new(Stub::default());
    let flatpak = Flatpak::new(runner.clone());
    let app = flatpak.installed().unwrap().remove(0);
    flatpak
        .perform(&app, Action::Remove, Installation::System)
        .unwrap();
    let calls = runner.calls.lock().unwrap();
    assert!(calls.iter().any(|args| args
        == &[
            "--system",
            "uninstall",
            "--app",
            "-y",
            "--noninteractive",
            "org.gimp.GIMP"
        ]));
}
