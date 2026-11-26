use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

use tungstenite::{Message, WebSocket};

use super::protocol::{active, answer, challenge, identified, identify, request};
use super::setup::Server;
use crate::context::Unavailable;

const TIMEOUT: Duration = Duration::from_millis(500);
const OUTPUTS: [&str; 2] = ["GetRecordStatus", "GetStreamStatus"];

pub fn capturing(server: &Server) -> Result<bool, Unavailable> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, server.port));
    let stream =
        TcpStream::connect_timeout(&address, TIMEOUT).map_err(|_| Unavailable::Unreachable)?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .and_then(|()| stream.set_write_timeout(Some(TIMEOUT)))
        .map_err(|_| Unavailable::Unreachable)?;
    let url = format!("ws://127.0.0.1:{}", server.port);
    let (mut socket, _) = tungstenite::client(url, stream).map_err(|_| Unavailable::Unreachable)?;
    let hello = text(&mut socket).ok_or(Unavailable::Unreachable)?;
    let authentication =
        challenge(&hello)
            .ok_or(Unavailable::Unreachable)?
            .map(|(salt, challenge)| {
                answer(
                    server.password.as_deref().unwrap_or_default(),
                    &salt,
                    &challenge,
                )
            });
    send(&mut socket, identify(authentication))?;
    if !text(&mut socket).is_some_and(|reply| identified(&reply)) {
        return Err(Unavailable::Unauthorized);
    }
    let mut capturing = false;
    for kind in OUTPUTS {
        send(&mut socket, request(kind))?;
        let reply = text(&mut socket).ok_or(Unavailable::Unreachable)?;
        capturing |= active(&reply, kind).ok_or(Unavailable::Unreachable)?;
    }
    let _ = socket.close(None);
    Ok(capturing)
}

fn send(socket: &mut WebSocket<TcpStream>, text: String) -> Result<(), Unavailable> {
    socket
        .send(Message::text(text))
        .map_err(|_| Unavailable::Unreachable)
}

fn text(socket: &mut WebSocket<TcpStream>) -> Option<String> {
    loop {
        match socket.read().ok()? {
            Message::Text(text) => return Some(text.to_string()),
            Message::Close(_) => return None,
            _ => continue,
        }
    }
}
