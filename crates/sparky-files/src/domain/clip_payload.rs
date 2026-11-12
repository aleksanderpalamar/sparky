use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use crate::domain::clip::Clip;
use crate::domain::thumbnail_key::file_uri;

pub const GNOME: &str = "x-special/gnome-copied-files";
pub const URI_LIST: &str = "text/uri-list";
pub const KDE_CUT: &str = "application/x-kde-cutselection";
pub const TEXT: &str = "text/plain;charset=utf-8";

pub fn gnome(clip: &Clip) -> String {
    let verb = match clip {
        Clip::Copy(_) => "copy",
        Clip::Cut(_) => "cut",
    };
    std::iter::once(verb.to_owned())
        .chain(clip.paths().iter().map(|path| file_uri(path)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn uri_list(clip: &Clip) -> String {
    clip.paths()
        .iter()
        .map(|path| format!("{}\r\n", file_uri(path)))
        .collect()
}

pub fn plain(clip: &Clip) -> String {
    clip.paths()
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_gnome(text: &str) -> Option<Clip> {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let verb = lines.next()?;
    let paths: Vec<PathBuf> = lines.filter_map(path_of).collect();
    let clip = match verb {
        "copy" => Clip::Copy(paths),
        "cut" => Clip::Cut(paths),
        _ => return None,
    };
    (!clip.paths().is_empty()).then_some(clip)
}

pub fn parse_uri_list(text: &str, cut: bool) -> Option<Clip> {
    let paths: Vec<PathBuf> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(path_of)
        .collect();
    if paths.is_empty() {
        return None;
    }
    Some(match cut {
        true => Clip::Cut(paths),
        false => Clip::Copy(paths),
    })
}

pub fn path_of(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let rest = rest.strip_prefix("localhost").unwrap_or(rest);
    if !rest.starts_with('/') {
        return None;
    }
    decode(rest).map(|bytes| PathBuf::from(OsString::from_vec(bytes)))
}

fn decode(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if byte != b'%' {
            decoded.push(byte);
            index += 1;
            continue;
        }
        let pair = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
        decoded.push(u8::from_str_radix(pair, 16).ok()?);
        index += 3;
    }
    (!decoded.contains(&0)).then_some(decoded)
}
