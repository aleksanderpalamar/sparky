use std::fs;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::path::Path;
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type};

use crate::domain::telemetry::{Ping, Sample};

pub fn sample(interface: String, gateway: Option<Ipv4Addr>) -> Result<Sample, String> {
    let (received, sent) = counters(Path::new("/sys/class/net"), &interface)
        .map_err(|error| format!("Contadores da rede: {error}"))?;
    let ping = gateway.map_or(Ping::Unavailable, probe);
    Ok(Sample {
        interface,
        at: Instant::now(),
        received,
        sent,
        ping,
    })
}

fn counters(base: &Path, interface: &str) -> io::Result<(u64, u64)> {
    if interface.is_empty()
        || interface == "."
        || interface == ".."
        || !interface
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "interface inválida",
        ));
    }
    let stats = base.join(interface).join("statistics");
    let read = |name| -> io::Result<u64> {
        fs::read_to_string(stats.join(name))?
            .trim()
            .parse()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    };
    Ok((read("rx_bytes")?, read("tx_bytes")?))
}

fn probe(gateway: Ipv4Addr) -> Ping {
    let Ok(socket) = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4)) else {
        return Ping::Unavailable;
    };
    let socket: UdpSocket = socket.into();
    if socket
        .set_read_timeout(Some(Duration::from_millis(600)))
        .is_err()
    {
        return Ping::Unavailable;
    }
    let sequence = std::process::id() as u16;
    let packet = echo(sequence);
    let started = Instant::now();
    if socket
        .send_to(&packet, SocketAddrV4::new(gateway, 0))
        .is_err()
    {
        return Ping::Unavailable;
    }
    let mut reply = [0; 64];
    loop {
        match socket.recv_from(&mut reply) {
            Ok((size, _)) if echo_reply(&reply[..size], sequence) => {
                return Ping::Reply(started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32);
            }
            Ok(_) => continue,
            Err(error)
                if error.kind() == io::ErrorKind::TimedOut
                    || error.kind() == io::ErrorKind::WouldBlock =>
            {
                return Ping::Lost;
            }
            Err(_) => return Ping::Unavailable,
        }
    }
}

fn echo(sequence: u16) -> [u8; 8] {
    let mut packet = [8, 0, 0, 0, 0, 0, (sequence >> 8) as u8, sequence as u8];
    let words: u32 = packet
        .chunks_exact(2)
        .map(|word| u32::from(u16::from_be_bytes([word[0], word[1]])))
        .sum();
    let checksum = !(words as u16);
    packet[2..4].copy_from_slice(&checksum.to_be_bytes());
    packet
}

fn echo_reply(packet: &[u8], sequence: u16) -> bool {
    let offset = if packet.first().is_some_and(|byte| byte >> 4 == 4) {
        usize::from(packet[0] & 0x0f) * 4
    } else {
        0
    };
    packet.get(offset) == Some(&0)
        && packet.get(offset + 6..offset + 8) == Some(sequence.to_be_bytes().as_slice())
}

#[cfg(test)]
mod tests {
    use super::{counters, echo, echo_reply};
    use std::fs;

    #[test]
    fn le_contadores_e_rejeita_escape_do_sysfs() {
        let dir = tempfile::tempdir().unwrap();
        let stats = dir.path().join("eth0/statistics");
        fs::create_dir_all(&stats).unwrap();
        fs::write(stats.join("rx_bytes"), "123\n").unwrap();
        fs::write(stats.join("tx_bytes"), "456\n").unwrap();
        assert_eq!(counters(dir.path(), "eth0").unwrap(), (123, 456));
        assert!(counters(dir.path(), "../fora").is_err());
    }

    #[test]
    fn pacote_icmp_identifica_resposta_da_amostra() {
        let request = echo(42);
        assert_eq!(request[0], 8);
        let mut reply = request;
        reply[0] = 0;
        assert!(echo_reply(&reply, 42));
        assert!(!echo_reply(&reply, 43));
    }
}
