use std::time::Duration;

use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::winit::{self, WinitEvent, WinitGraphicsBackend};
use smithay::desktop::layer_map_for_output;
use smithay::output::{Mode, Output, PhysicalProperties, Subpixel};
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::winit::dpi::LogicalSize;
use smithay::reexports::winit::platform::wayland::WindowAttributesExtWayland;
use smithay::reexports::winit::window::Window as HostWindow;
use smithay::utils::{Rectangle, Transform};

use crate::render::output::output_elements;
use crate::state::Sparky;

const REFRESH: i32 = 60_000;
const APP_ID: &str = "sparky-nested";

pub fn init(event_loop: &mut EventLoop<'static, Sparky>, state: &mut Sparky) -> Result<(), Box<dyn std::error::Error>> {
    let attributes = HostWindow::default_attributes()
        .with_inner_size(LogicalSize::new(1600.0, 900.0))
        .with_title("Sparky")
        .with_name(APP_ID, APP_ID)
        .with_visible(true);
    let (mut backend, winit) = winit::init_from_attributes::<GlesRenderer>(attributes)?;
    let mode = Mode {
        size: backend.window_size(),
        refresh: REFRESH,
    };
    let output = Output::new(
        "winit".to_owned(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Sparky".into(),
            model: "Aninhado".into(),
        },
    );
    let _global = output.create_global::<Sparky>(&state.display_handle);
    output.change_current_state(Some(mode), Some(Transform::Flipped180), None, Some((0, 0).into()));
    output.set_preferred(mode);
    state.space.map_output(&output, (0, 0));
    state.sync_outputs();
    let mut damage = OutputDamageTracker::from_output(&output);
    event_loop.handle().insert_source(winit, move |event, _, state| match event {
        WinitEvent::Resized { size, .. } => {
            output.change_current_state(Some(Mode { size, refresh: REFRESH }), None, None, None);
            state.sync_outputs();
            state.relayout();
        }
        WinitEvent::Input(event) => state.process_input_event(event),
        WinitEvent::Redraw => redraw(state, &mut backend, &output, &mut damage),
        WinitEvent::CloseRequested => state.loop_signal.stop(),
        WinitEvent::Focus(_) => {}
    })?;
    Ok(())
}

fn redraw(state: &mut Sparky, backend: &mut WinitGraphicsBackend<GlesRenderer>, output: &Output, damage: &mut OutputDamageTracker) {
    let size = backend.window_size();
    let clear = state.colors.background;
    let rendered = match backend.bind() {
        Ok((renderer, mut framebuffer)) => {
            let elements = output_elements(state, renderer, output);
            damage.render_output(renderer, &mut framebuffer, 0, &elements, clear).is_ok()
        }
        Err(error) => {
            tracing::warn!("sem framebuffer: {error}");
            false
        }
    };
    if rendered && let Err(error) = backend.submit(Some(&[Rectangle::from_size(size)])) {
        tracing::warn!("quadro não enviado: {error}");
    }
    send_frames(state, output);
    backend.window().request_redraw();
}

fn send_frames(state: &Sparky, output: &Output) {
    let time = state.start_time.elapsed();
    for window in state.space.elements() {
        window.send_frame(output, time, Some(Duration::ZERO), |_, _| Some(output.clone()));
    }
    for layer in layer_map_for_output(output).layers() {
        layer.send_frame(output, time, Some(Duration::ZERO), |_, _| Some(output.clone()));
    }
}
