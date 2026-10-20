use std::process::ExitCode;

use sparky_ipc::codec::encode;
use sparky_ipc::command::parse;
use sparky_ipc::{Client, Reply, Request};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("sparky-msg: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &[String]) -> Result<(), String> {
    let request = parse(arguments).map_err(|error| error.to_string())?;
    let mut client = Client::from_env().map_err(|error| error.to_string())?;
    if request == Request::Subscribe {
        let events = client.subscribe().map_err(|error| error.to_string())?;
        for event in events {
            print(&Reply::Event { event })?;
        }
        return Ok(());
    }
    let reply = client.request(&request).map_err(|error| error.to_string())?;
    print(&reply)
}

fn print(reply: &Reply) -> Result<(), String> {
    let line = encode(reply).map_err(|error| error.to_string())?;
    print!("{line}");
    Ok(())
}
