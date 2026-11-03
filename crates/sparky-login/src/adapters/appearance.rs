use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use nix::unistd::{Gid, Group, Uid, chown, getgroups, setegid, seteuid, setgroups};
use sparky_login_core::appearance::{COLORS, WALLPAPER};

use crate::adapters::identity::groups;
use crate::domain::appearance::{sources, wallpaper};
use crate::domain::environment::Account;
use crate::ports::appearance::Appearance;

const CONFIG_LIMIT: u64 = 1024 * 1024;
const COLORS_LIMIT: u64 = 64 * 1024;
const WALLPAPER_LIMIT: u64 = 32 * 1024 * 1024;
const SHARED: u32 = 0o640;
const READER: &str = "sparky-login";
const TEMPORARY: &str = "tmp";

pub struct Snapshot {
    pub colors: Option<Vec<u8>>,
    pub wallpaper: Option<Vec<u8>>,
}

pub struct UserAppearance;

impl Appearance for UserAppearance {
    fn capture(&self, account: &Account, environment: &[(String, String)], folder: &Path) {
        let reader = Group::from_name(READER)
            .ok()
            .flatten()
            .map(|group| group.gid);
        let result = as_user(account, || gather(account, environment)).and_then(|snapshot| {
            store(folder, &snapshot, reader).map_err(|error| error.to_string())
        });
        if let Err(error) = result {
            eprintln!("sparky-login: aparência da tela de login: {error}");
        }
    }
}

fn gather(account: &Account, environment: &[(String, String)]) -> Snapshot {
    let config_home = environment
        .iter()
        .find(|(name, _)| name == "XDG_CONFIG_HOME")
        .map(|(_, value)| value.as_str());
    let sources = sources(&account.home, config_home);
    let config = read_limited(&sources.config, CONFIG_LIMIT)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok());
    let chosen = wallpaper(config.as_deref(), &account.home);
    Snapshot {
        colors: read_limited(&sources.colors, COLORS_LIMIT).ok(),
        wallpaper: chosen.and_then(|path| read_limited(&path, WALLPAPER_LIMIT).ok()),
    }
}

pub fn read_limited(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(io::Error::other(format!(
            "{} não é uma imagem aceita",
            path.display()
        )));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    match bytes.len() as u64 > limit {
        true => Err(io::Error::other(format!(
            "{} passou de {limit} bytes",
            path.display()
        ))),
        false => Ok(bytes),
    }
}

pub fn store(folder: &Path, snapshot: &Snapshot, reader: Option<Gid>) -> io::Result<()> {
    fs::create_dir_all(folder)?;
    for (name, contents) in [(COLORS, &snapshot.colors), (WALLPAPER, &snapshot.wallpaper)] {
        let path = folder.join(name);
        match contents {
            Some(bytes) => replace(&path, bytes, reader)?,
            None => remove(&path)?,
        }
    }
    Ok(())
}

fn replace(path: &Path, bytes: &[u8], reader: Option<Gid>) -> io::Result<()> {
    let temporary = path.with_extension(TEMPORARY);
    let _ = fs::remove_file(&temporary);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(SHARED)
        .open(&temporary)?;
    io::Write::write_all(&mut file, bytes)?;
    fs::set_permissions(&temporary, fs::Permissions::from_mode(SHARED))?;
    if let Some(group) = reader {
        chown(&temporary, None, Some(group)).map_err(io::Error::from)?;
    }
    fs::rename(&temporary, path)
}

fn remove(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

fn as_user<T>(account: &Account, job: impl FnOnce() -> T) -> Result<T, String> {
    let saved = getgroups().map_err(|error| error.to_string())?;
    let entered = groups(account).and_then(|list| enter(account, &list));
    let result = entered.map(|()| job());
    leave(&saved)?;
    result
}

fn enter(account: &Account, list: &[Gid]) -> Result<(), String> {
    setgroups(list).map_err(|error| format!("grupos: {error}"))?;
    setegid(Gid::from_raw(account.gid)).map_err(|error| format!("gid: {error}"))?;
    seteuid(Uid::from_raw(account.uid)).map_err(|error| format!("uid: {error}"))
}

fn leave(saved: &[Gid]) -> Result<(), String> {
    seteuid(Uid::from_raw(0)).map_err(|error| format!("voltar ao root: {error}"))?;
    setegid(Gid::from_raw(0)).map_err(|error| format!("voltar ao grupo root: {error}"))?;
    setgroups(saved).map_err(|error| format!("voltar aos grupos: {error}"))
}
