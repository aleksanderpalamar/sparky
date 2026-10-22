use std::io::IsTerminal;
use std::process::ExitCode;
use std::sync::{Arc, atomic::AtomicU8};

use polkit_sparky::domain::Outcome;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.as_slice() {
        [] => daemon(),
        [argument] if argument == "--authenticate" => {
            prompt().map(|outcome| ExitCode::from(outcome.code()))
        }
        [argument] if argument == "--help" => {
            println!("uso: polkit-sparky\nAgente nativo de autenticação da sessão Sparky.");
            return ExitCode::SUCCESS;
        }
        _ => Err("uso: polkit-sparky".to_owned()),
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("polkit-sparky: {error}");
            ExitCode::from(Outcome::Denied.code())
        }
    }
}

fn daemon() -> Result<ExitCode, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime
        .block_on(polkit_sparky::service::daemon::run())
        .map_err(|error| error.to_string())?;
    Ok(ExitCode::SUCCESS)
}

fn prompt() -> Result<Outcome, String> {
    if std::io::stdin().is_terminal() {
        return Err("A janela de autenticação exige um canal privado".to_owned());
    }
    let (request, cancellation) = polkit_sparky::prompt::input::read()?;
    let result = Arc::new(AtomicU8::new(Outcome::Cancelled.code()));
    let backend = Arc::new(polkit_sparky::native::Native::default());
    polkit_sparky::prompt::run(request, cancellation, backend, result.clone())?;
    Ok(polkit_sparky::prompt::result(&result))
}
