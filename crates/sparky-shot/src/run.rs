use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use sparky_ipc::{CaptureArea, OutputInfo};
use sparky_tiling::Rect;

use crate::adapters::desktop::{NotifySend, WlCopy, screenshot_dir};
use crate::adapters::ipc::{SparkyCapturer, focused_output};
use crate::domain::delay::Delay;
use crate::domain::mode::{Invocation, Mode};
use crate::domain::pictures::file_name;
use crate::flow::{self, Ports};
use crate::picker::{self, Choice};
use crate::overlay;

const SETTLE: Duration = Duration::from_millis(200);

pub fn run(invocation: Invocation) -> Result<(), String> {
    let Some(choice) = choice(invocation) else {
        return Ok(());
    };
    let ports = Ports {
        capturer: &SparkyCapturer,
        clipboard: &WlCopy,
        notifier: &NotifySend,
    };
    let area = match area(choice.mode) {
        Ok(Some(area)) => area,
        Ok(None) => return Ok(()),
        Err(error) => return failed(&ports, error),
    };
    thread::sleep(SETTLE + choice.delay.duration());
    match screenshot_path() {
        Ok(path) => flow::take(&area, &path, &ports),
        Err(error) => failed(&ports, error),
    }
}

fn choice(invocation: Invocation) -> Option<Choice> {
    match invocation {
        Invocation::Capture(mode) => Some(immediate(mode)),
        Invocation::Picker => picker::choose(),
    }
}

fn immediate(mode: Mode) -> Choice {
    Choice {
        mode,
        delay: Delay::Immediate,
    }
}

fn area(mode: Mode) -> Result<Option<CaptureArea>, String> {
    match mode {
        Mode::Window => Ok(Some(CaptureArea::Window)),
        Mode::Output => Ok(Some(CaptureArea::Output { output: None })),
        Mode::Region => region(),
    }
}

fn region() -> Result<Option<CaptureArea>, String> {
    let output = focused_output().ok_or("nenhum monitor em foco: o sparky está rodando?")?;
    Ok(overlay::select(output.width, output.height).map(|rect| region_of(output, rect)))
}

fn region_of(output: OutputInfo, rect: Rect) -> CaptureArea {
    CaptureArea::Region {
        output: Some(output.name),
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn screenshot_path() -> Result<PathBuf, String> {
    Ok(screenshot_dir()?.join(file_name(&chrono::Local::now().naive_local())))
}

fn failed(ports: &Ports<'_>, error: String) -> Result<(), String> {
    ports.notifier.failed(&error);
    Err(error)
}

#[cfg(test)]
mod tests {
    use super::{area, immediate, region_of};
    use crate::domain::delay::Delay;
    use crate::domain::mode::Mode;
    use sparky_ipc::{CaptureArea, OutputInfo};
    use sparky_tiling::Rect;

    #[test]
    fn pelo_atalho_a_captura_e_imediata() {
        assert_eq!(immediate(Mode::Window).delay, Delay::Immediate);
    }

    #[test]
    fn janela_e_monitor_nao_precisam_de_selecao() {
        assert_eq!(area(Mode::Window), Ok(Some(CaptureArea::Window)));
        assert_eq!(area(Mode::Output), Ok(Some(CaptureArea::Output { output: None })));
    }

    #[test]
    fn a_regiao_vai_para_o_monitor_onde_foi_desenhada() {
        let output = OutputInfo { name: "DP-1".to_owned(), x: 1920, y: 0, width: 2560, height: 1440, focused: true };
        assert_eq!(
            region_of(output, Rect::new(10, 20, 300, 400)),
            CaptureArea::Region { output: Some("DP-1".to_owned()), x: 10, y: 20, width: 300, height: 400 }
        );
    }
}
