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
use smithay::utils::{Physical, Rectangle, Size, Transform};

use crate::backend::Backend;
use crate::backend::host_cursor::{HostCursor, host_cursor};
use crate::render::output::output_elements;
use crate::state::Sparky;

const REFRESH: i32 = 60_000;
const APP_ID: &str = "sparky-nested";

pub struct Nested {
    pub graphics: WinitGraphicsBackend<GlesRenderer>,
    pub output: Output,
    pub damage: OutputDamageTracker,
    pub cursor: Option<HostCursor>,
}

pub fn init(event_loop: &mut EventLoop<'static, Sparky>, state: &mut Sparky) -> Result<(), Box<dyn std::error::Error>> {
    let attributes = HostWindow::default_attributes()
        .with_inner_size(LogicalSize::new(1600.0, 900.0))
        .with_title("Sparky")
        .with_name(APP_ID, APP_ID)
        .with_visible(true);
    let (mut graphics, events) = winit::init_from_attributes::<GlesRenderer>(attributes)?;
    let output = nested_output(state, graphics.window_size());
    state.publish_formats(graphics.renderer());
    let damage = OutputDamageTracker::from_output(&output);
    state.backend = Backend::Nested(Box::new(Nested { graphics, output, damage, cursor: None }));
    event_loop.handle().insert_source(events, |event, _, state| match event {
        WinitEvent::Resized { size, .. } => state.resize_nested(size),
        WinitEvent::Input(event) => state.process_input_event(event),
        WinitEvent::Redraw => {
            state.with_nested(redraw);
        }
        WinitEvent::CloseRequested => state.loop_signal.stop(),
        WinitEvent::Focus(_) => {}
    })?;
    Ok(())
}

fn nested_output(state: &mut Sparky, size: Size<i32, Physical>) -> Output {
    let mode = Mode { size, refresh: REFRESH };
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
    output
}

impl Sparky {
    fn resize_nested(&mut self, size: Size<i32, Physical>) {
        if let Backend::Nested(nested) = &self.backend {
            nested
                .output
                .change_current_state(Some(Mode { size, refresh: REFRESH }), None, None, None);
        }
        self.sync_outputs();
        self.relayout();
    }
}

fn redraw(state: &mut Sparky, nested: &mut Nested) {
    let size = nested.graphics.window_size();
    let clear = state.colors.background;
    let rendered = match nested.graphics.bind() {
        Ok((renderer, mut framebuffer)) => {
            let elements = output_elements(state, renderer, &nested.output);
            nested
                .damage
                .render_output(renderer, &mut framebuffer, 0, &elements, clear)
                .is_ok()
        }
        Err(error) => {
            tracing::warn!("sem framebuffer: {error}");
            false
        }
    };
    if rendered && let Err(error) = nested.graphics.submit(Some(&[Rectangle::from_size(size)])) {
        tracing::warn!("quadro não enviado: {error}");
    }
    send_frames(state, &nested.output);
    apply_cursor(state, nested);
    nested.graphics.window().request_redraw();
}

fn apply_cursor(state: &Sparky, nested: &mut Nested) {
    let wanted = host_cursor(&state.cursor_status);
    if nested.cursor == Some(wanted) {
        return;
    }
    let window = nested.graphics.window();
    match wanted {
        HostCursor::Shape(icon) => {
            window.set_cursor_visible(true);
            window.set_cursor(icon);
        }
        HostCursor::Hidden => window.set_cursor_visible(false),
    }
    nested.cursor = Some(wanted);
}

pub fn send_frames(state: &Sparky, output: &Output) {
    let time = state.start_time.elapsed();
    for window in state.space.elements() {
        window.send_frame(output, time, Some(Duration::ZERO), |_, _| Some(output.clone()));
    }
    for layer in layer_map_for_output(output).layers() {
        layer.send_frame(output, time, Some(Duration::ZERO), |_, _| Some(output.clone()));
    }
}
