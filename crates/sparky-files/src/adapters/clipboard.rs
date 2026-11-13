use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

use crate::domain::clip::{Clip, ClipOffer, Pasted};
use crate::domain::clip_payload::{
    GNOME, KDE_CUT, TEXT, URI_LIST, gnome, parse_gnome, parse_uri_list, plain, uri_list,
};
use crate::ports::FileClipboard;

pub const SERVE: &str = "--serve-clipboard";
const READ_LIMIT: Duration = Duration::from_secs(2);

pub struct WaylandClipboard;

impl FileClipboard for WaylandClipboard {
    fn publish(&self, offer: &ClipOffer) -> io::Result<()> {
        spawn_server(&entries(offer))
    }

    fn clear(&self) -> io::Result<()> {
        spawn_server(&Value::Array(Vec::new()))
    }

    fn read(&self) -> Pasted {
        let Some(types) = paste(&["--list-types"]) else {
            return Pasted::Unavailable;
        };
        let types = match types {
            Ok(types) => types,
            Err(()) => return Pasted::Other,
        };
        let offered = |mime: &str| types.lines().any(|line| line.trim() == mime);
        let text = |mime: &str| paste(&["--no-newline", "--type", mime]).and_then(Result::ok);
        let clip = match (offered(GNOME), offered(URI_LIST)) {
            (true, _) => text(GNOME).and_then(|payload| parse_gnome(&payload)),
            (false, true) => {
                let cut = offered(KDE_CUT) && text(KDE_CUT).is_some_and(|mark| mark.trim() == "1");
                text(URI_LIST).and_then(|payload| parse_uri_list(&payload, cut))
            }
            (false, false) => None,
        };
        clip.map_or(Pasted::Other, Pasted::Files)
    }
}

pub fn entries(offer: &ClipOffer) -> Value {
    let clip = &offer.clip;
    let mut entries = vec![
        json!({"mime": GNOME, "text": gnome(clip)}),
        json!({"mime": URI_LIST, "text": uri_list(clip)}),
        json!({"mime": TEXT, "text": plain(clip)}),
        json!({"mime": "UTF8_STRING", "text": plain(clip)}),
    ];
    if matches!(clip, Clip::Cut(_)) {
        entries.push(json!({"mime": KDE_CUT, "text": "1"}));
    }
    if let Some((path, mime)) = &offer.image {
        entries.push(json!({"mime": mime.as_str(), "file": path}));
    }
    Value::Array(entries)
}

fn spawn_server(entries: &Value) -> io::Result<()> {
    let mut child = Command::new(std::env::current_exe()?)
        .arg(SERVE)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut input) = child.stdin.take() {
        input.write_all(entries.to_string().as_bytes())?;
    }
    thread::spawn(move || child.wait());
    Ok(())
}

fn paste(arguments: &[&str]) -> Option<Result<String, ()>> {
    let child = Command::new("wl-paste")
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = sender.send(child.wait_with_output());
    });
    let output = receiver.recv_timeout(READ_LIMIT).ok()?.ok()?;
    Some(match output.status.success() {
        true => String::from_utf8(output.stdout).map_err(drop),
        false => Err(()),
    })
}
