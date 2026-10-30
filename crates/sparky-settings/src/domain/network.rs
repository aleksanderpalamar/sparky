mod draft;

pub use draft::{Draft, Field, Method};

use crate::ports::commands::CommandRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub uuid: String,
    pub interface: String,
    pub name: String,
    pub method: Method,
    pub address: String,
    pub gateway: String,
    pub dns: String,
}

#[derive(Debug, Clone, Default)]
pub struct Network {
    pub profiles: Vec<Profile>,
    pub selected: usize,
    pub draft: Option<Draft>,
    pub error: Option<String>,
}

impl Network {
    pub fn load(commands: &dyn CommandRunner) -> Self {
        let listing = match commands.run(
            "nmcli",
            &["-t", "-f", "UUID,DEVICE", "connection", "show", "--active"],
        ) {
            Ok(listing) => listing,
            Err(error) => {
                return Self {
                    error: Some(error.to_string()),
                    ..Self::default()
                };
            }
        };
        let profiles = listing
            .lines()
            .filter_map(|line| {
                let (uuid, interface) = line.rsplit_once(':')?;
                if interface == "lo" || interface == "--" || uuid.is_empty() {
                    return None;
                }
                read_profile(commands, uuid, interface)
            })
            .collect::<Vec<_>>();
        let draft = profiles.first().map(Draft::from);
        Self {
            profiles,
            draft,
            ..Self::default()
        }
    }

    pub fn select(&mut self, index: usize) {
        let Some(profile) = self.profiles.get(index) else {
            return;
        };
        self.selected = index;
        self.draft = Some(Draft::from(profile));
    }

    pub fn save(&self, commands: &dyn CommandRunner) -> Result<(), String> {
        let profile = self
            .profiles
            .get(self.selected)
            .ok_or("Nenhuma conexão ativa selecionada")?;
        let draft = self
            .draft
            .as_ref()
            .ok_or("Configuração IPv4 indisponível")?;
        let arguments = draft.arguments(&profile.uuid)?;
        let arguments = arguments.iter().map(String::as_str).collect::<Vec<_>>();
        commands
            .run("nmcli", &arguments)
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

fn read_profile(commands: &dyn CommandRunner, uuid: &str, interface: &str) -> Option<Profile> {
    let details = commands
        .run(
            "nmcli",
            &[
                "-e",
                "no",
                "-g",
                "connection.id,ipv4.method,ipv4.addresses,ipv4.gateway,ipv4.dns",
                "connection",
                "show",
                uuid,
            ],
        )
        .ok()?;
    let mut lines = details.lines();
    let name = lines.next()?.to_owned();
    let method = Method::parse(lines.next()?)?;
    let mut address = lines.next().unwrap_or_default().to_owned();
    let mut gateway = lines.next().unwrap_or_default().to_owned();
    let dns = lines.next().unwrap_or_default().to_owned();
    if method == Method::Auto {
        address = current(commands, interface, "IP4.ADDRESS").unwrap_or(address);
        gateway = current(commands, interface, "IP4.GATEWAY").unwrap_or(gateway);
    }
    Some(Profile {
        uuid: uuid.to_owned(),
        interface: interface.to_owned(),
        name,
        method,
        address,
        gateway,
        dns,
    })
}

fn current(commands: &dyn CommandRunner, interface: &str, field: &str) -> Option<String> {
    let output = commands
        .run("nmcli", &["-g", field, "device", "show", interface])
        .ok()?;
    output
        .lines()
        .next()
        .and_then(|line| line.split(" | ").next())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests;
