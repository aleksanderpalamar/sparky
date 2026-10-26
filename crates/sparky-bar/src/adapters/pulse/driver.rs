use std::cell::{Cell, RefCell};
use std::io::{ErrorKind, Read};
use std::os::{fd::AsRawFd, unix::net::UnixStream};
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Receiver,
};
use std::time::Duration;

use iced::futures::channel::mpsc::UnboundedSender;
use libpulse_binding::context::{Context, FlagSet, State, subscribe::InterestMaskSet};
use libpulse_binding::mainloop::{
    api::Mainloop as _,
    events::io,
    standard::{IterateResult, Mainloop},
};

use super::channel::{Signal, Wake};
use super::{commands, snapshot};
use crate::domain::audio::Audio;

pub fn run(
    reader: UnixStream,
    receiver: Receiver<Signal>,
    wake: Wake,
    stopped: Arc<AtomicBool>,
    output: UnboundedSender<Audio>,
) {
    let receiver = Rc::new(receiver);
    while !stopped.load(Ordering::Acquire) && !output.is_closed() {
        let result = connected(
            &reader,
            receiver.clone(),
            wake.clone(),
            &stopped,
            output.clone(),
        );
        if stopped.load(Ordering::Acquire) {
            return;
        }
        let error = result
            .err()
            .unwrap_or_else(|| "A conexão de áudio foi encerrada".to_owned());
        if output.unbounded_send(Audio::Unavailable(error)).is_err() {
            return;
        }
        if !reconnect_wait(&receiver) {
            return;
        }
    }
}

fn reconnect_wait(receiver: &Receiver<Signal>) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    loop {
        match receiver.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now())) {
            Ok(Signal::Request(_, reply)) => {
                let _ = reply.send(Err("O serviço de áudio está indisponível".to_owned()));
            }
            Ok(Signal::Shutdown) => return false,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => return true,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return false,
            _ => {}
        }
    }
}

fn connected(
    reader: &UnixStream,
    receiver: Rc<Receiver<Signal>>,
    wake: Wake,
    stopped: &AtomicBool,
    output: UnboundedSender<Audio>,
) -> Result<(), String> {
    let mut mainloop = Mainloop::new().ok_or("Não foi possível criar o cliente de áudio")?;
    let mut context =
        Context::new(&mainloop, "Sparky").ok_or("Não foi possível conectar o áudio")?;
    let state_wake = wake.clone();
    context.set_state_callback(Some(Box::new(move || {
        let _ = state_wake.send(Signal::State);
    })));
    let refresh_wake = wake.clone();
    context.set_subscribe_callback(Some(Box::new(move |_, _, _| {
        let _ = refresh_wake.send(Signal::Refresh);
    })));
    context
        .connect(None, FlagSet::NOAUTOSPAWN, None)
        .map_err(|_| "Não foi possível conectar ao PipeWire/PulseAudio".to_owned())?;
    let context = Rc::new(RefCell::new(context));
    let ended = Rc::new(Cell::new(false));
    let mut socket = reader.try_clone().map_err(|error| error.to_string())?;
    let callback_context = context.clone();
    let callback_end = ended.clone();
    let store = snapshot::Shared::default();
    let ready = Cell::new(false);
    let _event = mainloop
        .new_io_event(
            socket.as_raw_fd(),
            io::FlagSet::INPUT,
            Box::new(move |_, _, _| {
                drain(&mut socket);
                for signal in receiver.try_iter() {
                    let mut context = callback_context.borrow_mut();
                    match signal {
                        Signal::Shutdown => callback_end.set(true),
                        Signal::State => match context.get_state() {
                            State::Ready if !ready.replace(true) => {
                                context.subscribe(
                                    InterestMaskSet::SINK
                                        | InterestMaskSet::SOURCE
                                        | InterestMaskSet::SERVER,
                                    |_| {},
                                );
                                snapshot::refresh(&context, &store, &output);
                            }
                            State::Failed | State::Terminated => callback_end.set(true),
                            _ => {}
                        },
                        Signal::Refresh if context.get_state() == State::Ready => {
                            snapshot::refresh(&context, &store, &output)
                        }
                        Signal::Request(request, reply) if context.get_state() == State::Ready => {
                            commands::execute(&mut context, request, reply, wake.clone())
                        }
                        Signal::Request(_, reply) => {
                            let _ =
                                reply.send(Err("O serviço de áudio não está pronto".to_owned()));
                        }
                        _ => {}
                    }
                }
            }),
        )
        .ok_or("Não foi possível observar o serviço de áudio")?;
    while !ended.get() && !stopped.load(Ordering::Acquire) {
        match mainloop.iterate(true) {
            IterateResult::Success(_) => {}
            _ => break,
        }
    }
    context.borrow_mut().set_state_callback(None);
    context.borrow_mut().set_subscribe_callback(None);
    context.borrow_mut().disconnect();
    Ok(())
}

fn drain(socket: &mut UnixStream) {
    let mut buffer = [0; 128];
    loop {
        match socket.read(&mut buffer) {
            Ok(0) => return,
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::WouldBlock => return,
            Err(_) => return,
        }
    }
}
