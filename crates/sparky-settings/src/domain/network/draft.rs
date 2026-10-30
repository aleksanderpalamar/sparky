use std::net::Ipv4Addr;

use super::Profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Auto,
    Manual,
}

impl Method {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Method::Auto),
            "manual" => Some(Method::Manual),
            _ => None,
        }
    }

    fn keyword(self) -> &'static str {
        match self {
            Method::Auto => "auto",
            Method::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Address,
    Gateway,
    Dns,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    pub method: Method,
    pub address: String,
    pub gateway: String,
    pub dns: String,
}

impl From<&Profile> for Draft {
    fn from(profile: &Profile) -> Self {
        Self {
            method: profile.method,
            address: profile.address.clone(),
            gateway: profile.gateway.clone(),
            dns: profile.dns.clone(),
        }
    }
}

impl Draft {
    pub fn change(&mut self, field: Field, value: String) {
        match field {
            Field::Address => self.address = value,
            Field::Gateway => self.gateway = value,
            Field::Dns => self.dns = value,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.method == Method::Manual {
            self.validate_manual()?;
        }
        for dns in self
            .dns
            .split([',', ' ', ';'])
            .filter(|entry| !entry.is_empty())
        {
            dns.parse::<Ipv4Addr>()
                .map_err(|_| format!("Servidor DNS IPv4 inválido: {dns}"))?;
        }
        Ok(())
    }

    fn validate_manual(&self) -> Result<(), String> {
        let Some((address, prefix)) = self.address.trim().split_once('/') else {
            return Err("Informe o endereço IPv4 com prefixo, como 192.168.1.10/24".to_owned());
        };
        address
            .parse::<Ipv4Addr>()
            .map_err(|_| "Endereço IPv4 inválido".to_owned())?;
        let prefix: u8 = prefix
            .parse()
            .map_err(|_| "Prefixo IPv4 inválido".to_owned())?;
        if prefix > 32 {
            return Err("Prefixo IPv4 deve estar entre 0 e 32".to_owned());
        }
        if !self.gateway.trim().is_empty() {
            self.gateway
                .trim()
                .parse::<Ipv4Addr>()
                .map_err(|_| "Gateway IPv4 inválido".to_owned())?;
        }
        Ok(())
    }

    pub fn arguments(&self, uuid: &str) -> Result<Vec<String>, String> {
        self.validate()?;
        let address = if self.method == Method::Manual {
            self.address.trim()
        } else {
            ""
        };
        let gateway = if self.method == Method::Manual {
            self.gateway.trim()
        } else {
            ""
        };
        let dns = self.dns.replace([',', ';'], " ");
        Ok(vec![
            "connection".to_owned(),
            "modify".to_owned(),
            uuid.to_owned(),
            "ipv4.method".to_owned(),
            self.method.keyword().to_owned(),
            "ipv4.addresses".to_owned(),
            address.to_owned(),
            "ipv4.gateway".to_owned(),
            gateway.to_owned(),
            "ipv4.dns".to_owned(),
            dns.trim().to_owned(),
            "ipv4.ignore-auto-dns".to_owned(),
            (!dns.trim().is_empty()).to_string(),
        ])
    }
}
