use libpulse_binding::context::Context;
use libpulse_binding::volume::{ChannelVolumes, Volume};

use super::channel::{Reply, Signal, Wake};
use crate::domain::audio::{AudioRequest, Direction};

pub fn execute(context: &mut Context, request: AudioRequest, reply: Reply, wake: Wake) {
    if !valid(&request) {
        let _ = reply.send(Err("Controle de áudio inválido".to_owned()));
        return;
    }
    let mut reply = Some(reply);
    let callback = Box::new(move |accepted| {
        let Some(reply) = reply.take() else {
            return;
        };
        let result = match accepted {
            true => Ok(()),
            false => Err("O serviço recusou o controle de áudio".to_owned()),
        };
        let _ = reply.send(result);
        let _ = wake.send(Signal::Refresh);
    });
    match request {
        AudioRequest::Volume {
            direction,
            name,
            channels,
            percent,
        } => {
            let mut volume = ChannelVolumes::default();
            volume.set(
                channels,
                Volume(u32::from(percent) * Volume::NORMAL.0 / 100),
            );
            match direction {
                Direction::Output => {
                    context
                        .introspect()
                        .set_sink_volume_by_name(&name, &volume, Some(callback));
                }
                Direction::Input => {
                    context
                        .introspect()
                        .set_source_volume_by_name(&name, &volume, Some(callback));
                }
            }
        }
        AudioRequest::Mute {
            direction,
            name,
            muted,
        } => match direction {
            Direction::Output => {
                context
                    .introspect()
                    .set_sink_mute_by_name(&name, muted, Some(callback));
            }
            Direction::Input => {
                context
                    .introspect()
                    .set_source_mute_by_name(&name, muted, Some(callback));
            }
        },
        AudioRequest::Default { direction, name } => match direction {
            Direction::Output => {
                context.set_default_sink(&name, callback);
            }
            Direction::Input => {
                context.set_default_source(&name, callback);
            }
        },
        AudioRequest::AppVolume {
            index,
            channels,
            percent,
        } => {
            let mut volume = ChannelVolumes::default();
            volume.set(
                channels,
                Volume(u32::from(percent) * Volume::NORMAL.0 / 100),
            );
            context
                .introspect()
                .set_sink_input_volume(index, &volume, Some(callback));
        }
    }
}

pub fn valid(request: &AudioRequest) -> bool {
    let name = match request {
        AudioRequest::AppVolume {
            channels, percent, ..
        } => return (1..=32).contains(channels) && *percent <= 150,
        AudioRequest::Volume {
            name,
            channels,
            percent,
            ..
        } => {
            if !(1..=32).contains(channels) || *percent > 150 {
                return false;
            }
            name
        }
        AudioRequest::Mute { name, .. } | AudioRequest::Default { name, .. } => name,
    };
    !name.is_empty() && !name.contains('\0')
}
