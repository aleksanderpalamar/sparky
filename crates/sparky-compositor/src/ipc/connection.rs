use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;

use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{Interest, Mode, PostAction};
use sparky_ipc::codec::{decode, encode};
use sparky_ipc::{Reply, Request};

use crate::ipc::lines::take_lines;
use crate::state::Sparky;

const CHUNK: usize = 4096;

impl Sparky {
    pub fn watch_connection(&mut self, stream: UnixStream) {
        if stream.set_nonblocking(true).is_err() {
            return;
        }
        let mut pending = Vec::new();
        let inserted = self
            .loop_handle
            .insert_source(Generic::new(stream, Interest::READ, Mode::Level), move |_, stream, state| {
                Ok(state.serve(stream, &mut pending))
            });
        if let Err(error) = inserted {
            tracing::warn!("conexão de IPC recusada: {error}");
        }
    }

    fn serve(&mut self, stream: &UnixStream, pending: &mut Vec<u8>) -> PostAction {
        if !read_available(stream, pending) {
            return PostAction::Remove;
        }
        for line in take_lines(pending) {
            let reply = match decode::<Request>(&line) {
                Ok(request) => self.answer_on(stream, request),
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

    fn answer_on(&mut self, stream: &UnixStream, request: Request) -> Reply {
        if request == Request::Subscribe
            && let (Some(server), Ok(copy)) = (self.ipc.as_mut(), stream.try_clone())
        {
            server.subscribers.push(copy);
        }
        self.answer(request)
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
