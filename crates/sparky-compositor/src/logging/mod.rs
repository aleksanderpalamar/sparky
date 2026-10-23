mod files;
mod queue;
mod worker;

use std::sync::Arc;

use tracing_subscriber::EnvFilter;

use crate::diagnostics::metrics::Metrics;

pub fn init(metrics: Arc<Metrics>) -> Option<worker::Guard> {
    let directory =
        files::state_directory(std::env::var_os("XDG_STATE_HOME"), std::env::var_os("HOME"));
    let (queue, guard) = match worker::start(directory.as_deref(), metrics) {
        Ok(worker) => worker,
        Err(error) => {
            eprintln!("sparky: não foi possível iniciar o diagnóstico: {error}");
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter())
                .with_writer(std::io::stderr)
                .try_init();
            return None;
        }
    };
    let initialized = tracing_subscriber::fmt()
        .with_env_filter(filter())
        .with_ansi(false)
        .with_writer(queue)
        .try_init();
    if let Err(error) = initialized {
        eprintln!("sparky: não foi possível registrar o logger: {error}");
        return None;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(panic = %info, "panic no Sparky");
        previous(info);
    }));
    tracing::info!(pid = std::process::id(), version = env!("CARGO_PKG_VERSION"), directory = ?directory, "início do diagnóstico");
    Some(guard)
}

fn filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod worker_tests;
