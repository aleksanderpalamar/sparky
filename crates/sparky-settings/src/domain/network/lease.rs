#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lease {
    pub address: String,
    pub gateway: String,
    pub dns: Vec<String>,
}

impl Lease {
    pub fn parse(output: &str) -> Self {
        let mut lines = output.lines().map(values);
        let mut first = || lines.next().unwrap_or_default().into_iter().next();
        let address = first().unwrap_or_default();
        let gateway = first().unwrap_or_default();
        let dns = lines.next().unwrap_or_default();
        Self {
            address,
            gateway,
            dns,
        }
    }
}

fn values(line: &str) -> Vec<String> {
    line.split(" | ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::Lease;

    #[test]
    fn le_endereco_gateway_e_dns_entregues_pelo_roteador() {
        let lease =
            Lease::parse("192.168.0.70/24 | 192.168.0.71/24\n192.168.0.1\n192.168.0.1 | 8.8.8.8\n");
        assert_eq!(lease.address, "192.168.0.70/24");
        assert_eq!(lease.gateway, "192.168.0.1");
        assert_eq!(lease.dns, ["192.168.0.1", "8.8.8.8"]);
    }

    #[test]
    fn campos_ausentes_ficam_vazios() {
        assert_eq!(Lease::parse("127.0.0.1/8\n\n\n").gateway, "");
        assert_eq!(Lease::parse(""), Lease::default());
    }
}
