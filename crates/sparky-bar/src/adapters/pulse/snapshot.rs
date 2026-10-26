use std::cell::RefCell;
use std::rc::Rc;

use iced::futures::channel::mpsc::UnboundedSender;
use libpulse_binding::{
    callbacks::ListResult,
    context::Context,
    volume::{ChannelVolumes, Volume},
};

use crate::domain::audio::{Audio, AudioDevice, AudioState};

#[derive(Default)]
pub struct Store {
    generation: u64,
    state: AudioState,
    completed: [bool; 3],
    failed: bool,
    published: Option<Audio>,
}

pub type Shared = Rc<RefCell<Store>>;

pub fn refresh(context: &Context, store: &Shared, output: &UnboundedSender<Audio>) {
    let generation = {
        let mut store = store.borrow_mut();
        store.generation = store.generation.wrapping_add(1);
        store.state = AudioState::default();
        store.completed = [false; 3];
        store.failed = false;
        store.generation
    };
    let server = store.clone();
    let sender = output.clone();
    context.introspect().get_server_info(move |info| {
        let mut store = server.borrow_mut();
        if store.generation != generation {
            return;
        }
        store.state.default_output = info.default_sink_name.as_deref().map(str::to_owned);
        store.state.default_input = info.default_source_name.as_deref().map(str::to_owned);
        store.completed[0] = true;
        publish(&mut store, &sender);
    });
    let sinks = store.clone();
    let sender = output.clone();
    context.introspect().get_sink_info_list(move |result| {
        let mut store = sinks.borrow_mut();
        if store.generation != generation {
            return;
        }
        match result {
            ListResult::Item(info) => {
                if let Some(device) = device(
                    info.name.as_deref(),
                    info.description.as_deref(),
                    &info.volume,
                    info.mute,
                ) {
                    store.state.outputs.push(device);
                }
            }
            ListResult::End => store.completed[1] = true,
            ListResult::Error => {
                store.failed = true;
                store.completed[1] = true;
            }
        }
        publish(&mut store, &sender);
    });
    let sources = store.clone();
    let sender = output.clone();
    context.introspect().get_source_info_list(move |result| {
        let mut store = sources.borrow_mut();
        if store.generation != generation {
            return;
        }
        match result {
            ListResult::Item(info) if info.monitor_of_sink.is_none() => {
                if let Some(device) = device(
                    info.name.as_deref(),
                    info.description.as_deref(),
                    &info.volume,
                    info.mute,
                ) {
                    store.state.inputs.push(device);
                }
            }
            ListResult::End => store.completed[2] = true,
            ListResult::Error => {
                store.failed = true;
                store.completed[2] = true;
            }
            _ => {}
        }
        publish(&mut store, &sender);
    });
}

fn publish(store: &mut Store, output: &UnboundedSender<Audio>) {
    if store.completed != [true; 3] {
        return;
    }
    store
        .state
        .outputs
        .sort_by(|left, right| left.label.cmp(&right.label));
    store
        .state
        .inputs
        .sort_by(|left, right| left.label.cmp(&right.label));
    let state = match store.failed {
        true => {
            Audio::Unavailable("Não foi possível consultar os dispositivos de áudio".to_owned())
        }
        false => Audio::Ready(store.state.clone()),
    };
    if store.published.as_ref() == Some(&state) {
        return;
    }
    store.published = Some(state.clone());
    let _ = output.unbounded_send(state);
}

fn device(
    name: Option<&str>,
    description: Option<&str>,
    volume: &ChannelVolumes,
    muted: bool,
) -> Option<AudioDevice> {
    let name = name.filter(|name| !name.is_empty() && !name.contains('\0'))?;
    Some(AudioDevice {
        name: name.to_owned(),
        label: description.unwrap_or(name).to_owned(),
        channels: volume.len(),
        volume: (u64::from(volume.avg().0) * 100 / u64::from(Volume::NORMAL.0)).min(150) as u16,
        muted,
    })
}
