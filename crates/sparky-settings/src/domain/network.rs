mod draft;
mod lease;
mod options;

pub use draft::{Draft, Field};
pub use lease::Lease;
pub use options::{AutoDns, Method};

use crate::ports::commands::CommandRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub uuid: String,
    pub interface: String,
    pub name: String,
    pub ipv4: Draft,
    pub lease: Lease,
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
        let draft = profiles.first().map(|profile| profile.ipv4.clone());
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
        self.draft = Some(profile.ipv4.clone());
    }

    pub fn switch(&mut self, method: Method) {
        let (Some(profile), Some(draft)) = (self.profiles.get(self.selected), self.draft.as_mut())
        else {
            return;
        };
        draft.switch(method, &profile.lease);
    }

    pub fn edit(&mut self, change: impl FnOnce(&mut Draft)) {
        if let Some(draft) = &mut self.draft {
            change(draft);
        }
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
        let arguments = draft.arguments(&profile.uuid, profile.ipv4.method)?;
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
                "connection.id,ipv4.method,ipv4.addresses,ipv4.gateway,ipv4.dns,ipv4.ignore-auto-dns",
                "connection",
                "show",
                uuid,
            ],
        )
        .ok()?;
    let mut lines = details.lines();
    let name = lines.next()?.to_owned();
    let method = Method::parse(lines.next()?)?;
    let address = lines.next().unwrap_or_default().to_owned();
    let gateway = lines.next().unwrap_or_default().to_owned();
    let dns = lines.next().unwrap_or_default();
    let auto_dns = lines
        .next()
        .and_then(AutoDns::parse)
        .unwrap_or(AutoDns::Use);
    let mut ipv4 = Draft {
        method,
        address,
        gateway,
        primary_dns: String::new(),
        secondary_dns: String::new(),
        auto_dns,
    };
    ipv4.set_servers(dns);
    Some(Profile {
        uuid: uuid.to_owned(),
        interface: interface.to_owned(),
        name,
        ipv4,
        lease: read_lease(commands, interface),
    })
}

fn read_lease(commands: &dyn CommandRunner, interface: &str) -> Lease {
    commands
        .run(
            "nmcli",
            &[
                "-g",
                "IP4.ADDRESS,IP4.GATEWAY,IP4.DNS",
                "device",
                "show",
                interface,
            ],
        )
        .map(|output| Lease::parse(&output))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
