use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ping {
    Reply(u32),
    Lost,
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub interface: String,
    pub at: Instant,
    pub received: u64,
    pub sent: u64,
    pub ping: Ping,
}

#[derive(Debug, Clone)]
pub struct Metrics {
    pub interface: String,
    pub received: u64,
    pub sent: u64,
    pub receive_rate: u64,
    pub send_rate: u64,
    pub ping_ms: Option<u32>,
    pub loss_percent: Option<u8>,
    at: Instant,
    history: VecDeque<bool>,
}

impl Metrics {
    pub fn begin(sample: Sample) -> Self {
        let mut metrics = Self {
            interface: sample.interface.clone(),
            received: sample.received,
            sent: sample.sent,
            receive_rate: 0,
            send_rate: 0,
            ping_ms: None,
            loss_percent: None,
            at: sample.at,
            history: VecDeque::with_capacity(10),
        };
        metrics.update_ping(sample.ping);
        metrics
    }

    pub fn update(&mut self, sample: Sample) {
        if self.interface != sample.interface {
            *self = Self::begin(sample);
            return;
        }
        let elapsed = sample.at.saturating_duration_since(self.at).as_millis();
        if elapsed > 0 {
            self.receive_rate = rate(sample.received, self.received, elapsed);
            self.send_rate = rate(sample.sent, self.sent, elapsed);
        }
        self.received = sample.received;
        self.sent = sample.sent;
        self.at = sample.at;
        self.update_ping(sample.ping);
    }

    fn update_ping(&mut self, ping: Ping) {
        match ping {
            Ping::Reply(milliseconds) => {
                self.ping_ms = Some(milliseconds);
                self.history.push_back(false);
            }
            Ping::Lost => {
                self.ping_ms = None;
                self.history.push_back(true);
            }
            Ping::Unavailable => return,
        }
        if self.history.len() > 10 {
            self.history.pop_front();
        }
        let lost = self.history.iter().filter(|lost| **lost).count();
        self.loss_percent = Some((lost * 100 / self.history.len()) as u8);
    }
}

fn rate(current: u64, previous: u64, elapsed_ms: u128) -> u64 {
    u128::from(current.saturating_sub(previous))
        .saturating_mul(1000)
        .checked_div(elapsed_ms)
        .unwrap_or(0)
        .min(u128::from(u64::MAX)) as u64
}

pub fn bytes(value: u64) -> String {
    let value = value as f64;
    match value {
        value if value >= 1_000_000_000.0 => format!("{:.2} GB", value / 1_000_000_000.0),
        value if value >= 1_000_000.0 => format!("{:.1} MB", value / 1_000_000.0),
        value if value >= 1_000.0 => format!("{:.1} kB", value / 1_000.0),
        _ => format!("{value:.0} B"),
    }
}

#[cfg(test)]
mod tests {
    use super::{Metrics, Ping, Sample, bytes};
    use std::time::{Duration, Instant};

    #[test]
    fn taxas_e_perda_usam_contadores_e_amostras_reais() {
        let at = Instant::now();
        let mut metrics = Metrics::begin(Sample {
            interface: "enp7s0".to_owned(),
            at,
            received: 100,
            sent: 100,
            ping: Ping::Reply(8),
        });
        metrics.update(Sample {
            interface: "enp7s0".to_owned(),
            at: at + Duration::from_secs(2),
            received: 590,
            sent: 470,
            ping: Ping::Lost,
        });
        assert_eq!(metrics.receive_rate, 245);
        assert_eq!(metrics.send_rate, 185);
        assert_eq!(metrics.loss_percent, Some(50));
        assert_eq!(bytes(metrics.receive_rate), "245 B");
    }

    #[test]
    fn troca_de_interface_reinicia_taxas_e_historico() {
        let at = Instant::now();
        let mut metrics = Metrics::begin(Sample {
            interface: "eth0".to_owned(),
            at,
            received: 1000,
            sent: 2000,
            ping: Ping::Lost,
        });
        metrics.update(Sample {
            interface: "wlan0".to_owned(),
            at: at + Duration::from_secs(1),
            received: 10,
            sent: 20,
            ping: Ping::Reply(5),
        });
        assert_eq!(metrics.receive_rate, 0);
        assert_eq!(metrics.loss_percent, Some(0));
    }
}
