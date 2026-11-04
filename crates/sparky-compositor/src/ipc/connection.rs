use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;

use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{Interest, Mode, PostAction};
use sparky_ipc::codec::{decode, encode};
use sparky_ipc::{IdleHold, Reply, Request};

use crate::ipc::lines::take_lines;
use crate::state::Sparky;

const CHUNK: usize = 4096;

#[derive(Default)]
struct Peer {
    pending: Vec<u8>,
    hold: IdleHold,
}

impl Sparky {
    pub fn watch_connection(&mut self, stream: UnixStream) {
        if stream.set_nonblocking(true).is_err() {
            return;
        }
        let mut peer = Peer::default();
        let inserted = self.loop_handle.insert_source(
            Generic::new(stream, Interest::READ, Mode::Level),
            move |_, stream, state| {
                let action = state.serve(stream, &mut peer);
                if matches!(action, PostAction::Remove) {
                    state.idle.hold(peer.hold, IdleHold::Free);
                }
                Ok(action)
            },
        );
        if let Err(error) = inserted {
            tracing::warn!("conexão de IPC recusada: {error}");
        }
    }

    fn serve(&mut self, stream: &UnixStream, peer: &mut Peer) -> PostAction {
        if !read_available(stream, &mut peer.pending) {
            return PostAction::Remove;
        }
        for line in take_lines(&mut peer.pending) {
            let reply = match decode::<Request>(&line) {
                Ok(request) => self.answer_on(stream, request, &mut peer.hold),
                Err(error) => Reply::Error {
                    message: error.to_string(),
                },
            };
            if !write_reply(stream, &reply) {
                return PostAction::Remove;
            }
        }
        PostAction::Continue
    }

    fn answer_on(&mut self, stream: &UnixStream, request: Request, hold: &mut IdleHold) -> Reply {
        if request == Request::Subscribe
            && let (Some(server), Ok(copy)) = (self.ipc.as_mut(), stream.try_clone())
        {
            server.subscribers.push(copy);
        }
        self.answer(request, hold)
    }
}

fn read_available(mut stream: &UnixStream, pending: &mut Vec<u8>) -> bool {
    let mut chunk = [0u8; CHUNK];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return false,
            Ok(read) => pending.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == ErrorKind::WouldBlock => return true,
            Err(_) => return false,
        }
    }
}

pub fn write_reply(mut stream: &UnixStream, reply: &Reply) -> bool {
    match encode(reply) {
        Ok(line) => stream.write_all(line.as_bytes()).is_ok(),
        Err(_) => false,
    }
}
