use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use futures_util::future::BoxFuture;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::watch;
use zeroize::Zeroizing;

use super::Runner;
use crate::domain::{Outcome, Request};

pub struct PromptProcess(pub PathBuf);

impl Runner for PromptProcess {
    fn run(
        &self,
        request: Request,
        cancellation: watch::Receiver<bool>,
    ) -> BoxFuture<'static, Outcome> {
        let program = self.0.clone();
        Box::pin(async move {
            match authenticate(program, request, cancellation).await {
                Ok(outcome) => outcome,
                Err(error) => {
                    eprintln!("polkit-sparky: não foi possível apresentar a autenticação: {error}");
                    Outcome::Denied
                }
            }
        })
    }
}

async fn authenticate(
    program: PathBuf,
    request: Request,
    mut cancellation: watch::Receiver<bool>,
) -> Result<Outcome, Box<dyn std::error::Error + Send + Sync>> {
    if *cancellation.borrow() {
        return Ok(Outcome::Cancelled);
    }
    let mut child = Command::new(program)
        .arg("--authenticate")
        .env_remove("POLKIT_DEBUG")
        .env_remove("RUST_LOG")
        .env_remove("G_MESSAGES_DEBUG")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()?;
    let mut input = child.stdin.take().ok_or("Canal privado indisponível")?;
    let mut packet = Zeroizing::new(serde_json::to_vec(&request)?);
    packet.push(b'\n');
    input.write_all(&packet).await?;
    input.flush().await?;
    drop(packet);
    let result = tokio::select! {
        result = child.wait() => result?,
        _ = cancellation.changed() => {
            drop(input);
            if tokio::time::timeout(Duration::from_secs(3), child.wait()).await.is_err() {
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
            return Ok(Outcome::Cancelled);
        }
    };
    Ok(Outcome::from_code(result.code()))
}
