use super::{components::NOTIFICATIONS, environment::Environment};
use crate::spawn::spawn;

pub fn launch(
    components: Vec<String>,
    apps: impl IntoIterator<Item = String>,
    environment: &Environment,
) {
    let notifications = components.iter().any(|command| command == NOTIFICATIONS);
    for command in components {
        tracing::info!("sessão abre {command}");
        spawn(&command, environment);
    }
    if notifications {
        wait(environment);
    }
    for command in apps {
        tracing::info!("sessão abre {command}");
        spawn(&command, environment);
    }
}

fn wait(environment: &Environment) {
    let result = std::process::Command::new(NOTIFICATIONS)
        .arg("--wait-ready")
        .envs(environment.variables())
        .stdin(std::process::Stdio::null())
        .status();
    match result {
        Ok(status) if status.success() => {}
        Ok(status) => tracing::warn!("notificações nativas indisponíveis: {status}"),
        Err(error) => tracing::warn!("não foi possível aguardar notificações: {error}"),
    }
}
