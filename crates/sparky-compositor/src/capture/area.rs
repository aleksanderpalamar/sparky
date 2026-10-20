use sparky_ipc::CaptureArea;
use sparky_tiling::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub output: String,
    pub region: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    NoOutput,
    NoWindow,
    EmptyRegion,
    Render(String),
    Write(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::NoOutput => write!(formatter, "monitor não encontrado"),
            CaptureError::NoWindow => write!(formatter, "nenhuma janela em foco"),
            CaptureError::EmptyRegion => write!(formatter, "a região não cobre nada da tela"),
            CaptureError::Render(message) => write!(formatter, "falha ao desenhar a captura: {message}"),
            CaptureError::Write(message) => write!(formatter, "falha ao gravar a captura: {message}"),
        }
    }
}

pub fn resolve(
    area: &CaptureArea,
    screens: &[(String, Rect)],
    focused: Option<&str>,
    window: Option<Rect>,
) -> Result<Target, CaptureError> {
    match area {
        CaptureArea::Output { output } => {
            let (name, screen) = screen_named(screens, output.as_deref().or(focused))?;
            Ok(Target {
                output: name,
                region: Rect::new(0, 0, screen.width, screen.height),
            })
        }
        CaptureArea::Window => {
            let window = window.ok_or(CaptureError::NoWindow)?;
            let (name, screen) = screen_holding(screens, window)?;
            clipped(name, screen, Rect::new(window.x - screen.x, window.y - screen.y, window.width, window.height))
        }
        CaptureArea::Region { output, x, y, width, height } => {
            let (name, screen) = screen_named(screens, output.as_deref().or(focused))?;
            clipped(name, screen, Rect::new(*x, *y, *width, *height))
        }
    }
}

fn screen_named(screens: &[(String, Rect)], name: Option<&str>) -> Result<(String, Rect), CaptureError> {
    let name = name.ok_or(CaptureError::NoOutput)?;
    screens
        .iter()
        .find(|(known, _)| known == name)
        .cloned()
        .ok_or(CaptureError::NoOutput)
}

fn screen_holding(screens: &[(String, Rect)], window: Rect) -> Result<(String, Rect), CaptureError> {
    screens
        .iter()
        .filter_map(|(name, screen)| screen.intersection(&window).map(|common| (name, screen, common)))
        .max_by_key(|(_, _, common)| common.width * common.height)
        .map(|(name, screen, _)| (name.clone(), *screen))
        .ok_or(CaptureError::NoOutput)
}

fn clipped(name: String, screen: Rect, relative: Rect) -> Result<Target, CaptureError> {
    let bounds = Rect::new(0, 0, screen.width, screen.height);
    let region = bounds.intersection(&relative).ok_or(CaptureError::EmptyRegion)?;
    Ok(Target { output: name, region })
}
