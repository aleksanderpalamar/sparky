use std::os::fd::AsFd;

use wayland_client::QueueHandle;
use wayland_client::protocol::{wl_buffer, wl_output, wl_shm, wl_surface};

use super::client::Client;
use super::events::{Events, constraints, layer_shell, layer_surface};
use super::fixture::Error;
use super::server::wait;

pub struct Bar {
    pub layer: layer_shell::Layer,
    pub anchor: layer_surface::Anchor,
    pub size: (u32, u32),
    pub exclusive: i32,
    pub input: Option<(i32, i32, i32, i32)>,
}

impl Bar {
    pub fn top() -> Self {
        Self {
            layer: layer_shell::Layer::Top,
            anchor: layer_surface::Anchor::Top
                | layer_surface::Anchor::Left
                | layer_surface::Anchor::Right,
            size: (0, 30),
            exclusive: 30,
            input: None,
        }
    }

    pub fn overlay(input: (i32, i32, i32, i32)) -> Self {
        Self {
            layer: layer_shell::Layer::Overlay,
            anchor: layer_surface::Anchor::Top | layer_surface::Anchor::Left,
            size: (100, 100),
            exclusive: -1,
            input: Some(input),
        }
    }
}

impl Client {
    pub fn layer(
        &mut self,
        bar: &Bar,
        output: Option<&wl_output::WlOutput>,
    ) -> Result<wl_surface::WlSurface, Error> {
        let qh = self.queue.handle();
        let surface = self.compositor.create_surface(&qh, ());
        let role = self.layers.get_layer_surface(
            &surface,
            output,
            bar.layer,
            "sparky-teste".into(),
            &qh,
            (),
        );
        role.set_anchor(bar.anchor);
        role.set_size(bar.size.0, bar.size.1);
        role.set_exclusive_zone(bar.exclusive);
        if let Some((x, y, width, height)) = bar.input {
            let region = self.compositor.create_region(&qh, ());
            region.add(x, y, width, height);
            surface.set_input_region(Some(&region));
        }
        surface.commit();
        let before = self.events.layer_sizes.len();
        wait(
            &mut self.queue,
            &mut self.events,
            |events| events.layer_sizes.len() > before,
            "camada sem configure",
        )?;
        let (width, height) = self.events.layer_sizes.last().copied().unwrap_or(bar.size);
        surface.attach(Some(&buffer(&self.shm, &qh, width, height)?), 0, 0);
        surface.commit();
        self.queue.roundtrip(&mut self.events)?;
        Ok(surface)
    }

    pub fn confine(&mut self) -> Result<(), Error> {
        let qh = self.queue.handle();
        let manager: constraints::ZwpPointerConstraintsV1 = self.globals.bind(&qh, 1..=1, ())?;
        manager.confine_pointer(
            &self.window,
            &self.pointer,
            None,
            constraints::Lifetime::Persistent,
            &qh,
            (),
        );
        self.window.commit();
        wait(
            &mut self.queue,
            &mut self.events,
            |events| events.confined,
            "ponteiro não confinado",
        )
    }
}

pub fn buffer(
    shm: &wl_shm::WlShm,
    qh: &QueueHandle<Events>,
    width: u32,
    height: u32,
) -> Result<wl_buffer::WlBuffer, Error> {
    let stride = width * 4;
    let length = stride * height;
    let file = tempfile::tempfile()?;
    file.set_len(u64::from(length))?;
    let pool = shm.create_pool(file.as_fd(), length as i32, qh, ());
    let created = pool.create_buffer(
        0,
        width as i32,
        height as i32,
        stride as i32,
        wl_shm::Format::Argb8888,
        qh,
        (),
    );
    pool.destroy();
    Ok(created)
}
