use std::sync::Arc;

use super::SystemStore;
use super::command::{CommandOutput, Runner};
use super::flatpak::Flatpak;
use super::native::Native;
use crate::domain::updates::UpdateSource;
use crate::ports::StoreBackend;

#[derive(Debug)]
struct Partial;

impl Runner for Partial {
    fn run(&self, program: &str, _: &[&str]) -> Result<String, String> {
        Err(format!("{program} unavailable"))
    }

    fn run_status(&self, program: &str, _: &[&str]) -> Result<CommandOutput, String> {
        if program != "checkupdates" {
            return Err(format!("{program} unavailable"));
        }
        Ok(CommandOutput {
            stdout: "mesa 24.1 -> 24.2\n".into(),
            stderr: String::new(),
            code: Some(0),
        })
    }
}

#[test]
fn falha_flatpak_preserva_atualizacoes_nativas() {
    let runner: Arc<dyn Runner> = Arc::new(Partial);
    let store = SystemStore {
        flatpak: Flatpak::new(runner.clone()),
        native: Native::from_release(runner, "ID=arch\n"),
    };
    let report = store.check_updates(false);
    assert_eq!(report.count(), 1);
    assert_eq!(
        report
            .groups
            .iter()
            .filter(|group| group.error.is_some())
            .count(),
        2
    );
    assert!(matches!(report.groups[2].source, UpdateSource::Native(_)));
}
