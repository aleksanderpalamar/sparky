use std::net::Ipv4Addr;

use super::Lease;
use super::options::{AutoDns, Method};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Address,
    Gateway,
    PrimaryDns,
    SecondaryDns,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    pub method: Method,
    pub address: String,
    pub gateway: String,
    pub primary_dns: String,
    pub secondary_dns: String,
    pub auto_dns: AutoDns,
}

impl Draft {
    pub fn change(&mut self, field: Field, value: String) {
        match field {
            Field::Address => self.address = value,
            Field::Gateway => self.gateway = value,
            Field::PrimaryDns => self.primary_dns = value,
            Field::SecondaryDns => self.secondary_dns = value,
        }
    }

    pub fn set_servers(&mut self, servers: &str) {
        let mut servers = split_servers(servers);
        self.primary_dns = servers.next().unwrap_or_default().to_owned();
        self.secondary_dns = servers.collect::<Vec<_>>().join(", ");
    }

    pub fn switch(&mut self, method: Method, lease: &Lease) {
        self.method = method;
        if method == Method::Auto {
            return;
        }
        fill(&mut self.address, &lease.address);
        fill(&mut self.gateway, &lease.gateway);
        if self.servers().next().is_none() {
            self.set_servers(&lease.dns.join(","));
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.method {
            Method::Manual => self.validate_manual()?,
            Method::Auto => self.validate_auto()?,
        }
        for dns in self.servers() {
            dns.parse::<Ipv4Addr>()
                .map_err(|_| crate::i18n::with("network-invalid-dns", "value", dns))?;
        }
        Ok(())
    }

    fn validate_manual(&self) -> Result<(), String> {
        let addresses = self.addresses();
        if addresses.is_empty() {
            return Err(crate::i18n::text("network-address-required"));
        }
        addresses.into_iter().try_for_each(validate_address)?;
        if !self.gateway.trim().is_empty() {
            self.gateway
                .trim()
                .parse::<Ipv4Addr>()
                .map_err(|_| crate::i18n::text("network-invalid-gateway"))?;
        }
        Ok(())
    }

    fn validate_auto(&self) -> Result<(), String> {
        if self.auto_dns == AutoDns::Ignore && self.servers().next().is_none() {
            return Err(crate::i18n::text("network-dns-required"));
        }
        Ok(())
    }

    fn addresses(&self) -> Vec<&str> {
        self.address
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .collect()
    }

    fn servers(&self) -> impl Iterator<Item = &str> {
        split_servers(&self.primary_dns).chain(split_servers(&self.secondary_dns))
    }

    pub fn arguments(&self, uuid: &str, saved: Method) -> Result<Vec<String>, String> {
        self.validate()?;
        let mut arguments = vec![
            "connection".to_owned(),
            "modify".to_owned(),
            uuid.to_owned(),
            "ipv4.method".to_owned(),
            self.method.keyword().to_owned(),
        ];
        if let Some((addresses, gateway)) = self.addressing(saved) {
            arguments.extend([
                "ipv4.addresses".to_owned(),
                addresses,
                "ipv4.gateway".to_owned(),
                gateway,
            ]);
        }
        arguments.extend([
            "ipv4.dns".to_owned(),
            self.servers().collect::<Vec<_>>().join(" "),
            "ipv4.ignore-auto-dns".to_owned(),
            self.auto_dns.keyword().to_owned(),
        ]);
        Ok(arguments)
    }

    fn addressing(&self, saved: Method) -> Option<(String, String)> {
        match (saved, self.method) {
            (_, Method::Manual) => {
                Some((self.addresses().join(", "), self.gateway.trim().to_owned()))
            }
            (Method::Manual, Method::Auto) => Some((String::new(), String::new())),
            (Method::Auto, Method::Auto) => None,
        }
    }
}

fn split_servers(text: &str) -> impl Iterator<Item = &str> {
    text.split([',', ' ', ';'])
        .filter(|entry| !entry.is_empty())
}

fn fill(field: &mut String, value: &str) {
    if field.trim().is_empty() {
        value.clone_into(field);
    }
}

fn validate_address(entry: &str) -> Result<(), String> {
    let Some((address, prefix)) = entry.split_once('/') else {
        return Err(crate::i18n::with("network-prefix-required", "value", entry));
    };
    address
        .parse::<Ipv4Addr>()
        .map_err(|_| crate::i18n::with("network-invalid-address", "value", entry))?;
    match prefix.parse::<u8>() {
        Ok(0..=32) => Ok(()),
        _ => Err(crate::i18n::with("network-invalid-prefix", "value", entry)),
    }
}

#[cfg(test)]
mod tests;
