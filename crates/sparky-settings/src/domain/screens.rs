use sparky_config::{OutputMode, OutputRule, OutputScale, Rotation, rule_for};
use sparky_ipc::{Hdr, OutputInfo, Vrr};

const COMMON_SCALES: [f64; 6] = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenChange {
    Mode(String),
    Scale(String),
    Rotation(String),
    Vrr(bool),
    Enabled(bool),
    Hdr(bool),
}

pub fn mode_options(info: &OutputInfo) -> Vec<String> {
    let mut labels: Vec<String> = Vec::new();
    for label in info.modes.iter().map(OutputMode::label) {
        if !labels.contains(&label) {
            labels.push(label);
        }
    }
    labels
}

pub fn scale_label(scale: OutputScale) -> String {
    format!("{}", (scale.factor() * 1000.0).round() / 1000.0)
}

pub fn scale_options(current: OutputScale) -> Vec<String> {
    let mut scales: Vec<OutputScale> = COMMON_SCALES
        .into_iter()
        .filter_map(OutputScale::from_factor)
        .collect();
    if !scales.contains(&current) {
        scales.push(current);
    }
    scales.sort_by_key(|scale| scale.units());
    scales.into_iter().map(scale_label).collect()
}

pub const fn rotation_label(rotation: Rotation) -> &'static str {
    match rotation {
        Rotation::Normal => "Normal",
        Rotation::Right => "90° à direita",
        Rotation::Inverted => "180°",
        Rotation::Left => "90° à esquerda",
    }
}

pub fn rotation_options() -> Vec<String> {
    Rotation::ALL
        .into_iter()
        .map(|rotation| rotation_label(rotation).to_owned())
        .collect()
}

pub fn changed(
    info: &OutputInfo,
    rules: &[OutputRule],
    outputs: &[OutputInfo],
    change: ScreenChange,
) -> Result<OutputRule, String> {
    let mut rule = rule_for(rules, &info.name)
        .cloned()
        .unwrap_or_else(|| OutputRule {
            scale: info.scale,
            ..OutputRule::new(&info.name)
        });
    match change {
        ScreenChange::Mode(label) => rule.mode = Some(known_mode(info, &label)?),
        ScreenChange::Scale(label) => rule.scale = scale(&label)?,
        ScreenChange::Rotation(label) => rule.rotation = rotation(&label)?,
        ScreenChange::Vrr(on) if info.vrr != Vrr::Unsupported => rule.vrr = Some(on),
        ScreenChange::Vrr(_) => return Err("este monitor não tem taxa variável".to_owned()),
        ScreenChange::Enabled(false) if last_enabled(outputs, &info.name) => {
            return Err("é a única tela ligada; desligá-la deixaria você sem imagem".to_owned());
        }
        ScreenChange::Enabled(on) => rule.enabled = on,
        ScreenChange::Hdr(on) if info.hdr != Hdr::Unsupported => rule.hdr = on,
        ScreenChange::Hdr(_) => {
            return Err("esta tela não suporta HDR".to_owned());
        }
    }
    Ok(rule)
}

pub fn last_enabled(outputs: &[OutputInfo], name: &str) -> bool {
    outputs
        .iter()
        .filter(|output| output.enabled)
        .all(|output| output.name == name)
}

fn known_mode(info: &OutputInfo, label: &str) -> Result<OutputMode, String> {
    let wanted = OutputMode::parse(label).ok_or_else(|| format!("resolução inválida: {label}"))?;
    info.modes
        .iter()
        .find(|mode| wanted.matches(mode))
        .copied()
        .ok_or_else(|| format!("{label} não está disponível neste monitor"))
}

fn scale(label: &str) -> Result<OutputScale, String> {
    label
        .trim()
        .replace(',', ".")
        .parse()
        .ok()
        .and_then(OutputScale::from_factor)
        .ok_or_else(|| format!("escala inválida: {label}; use de 0.5 a 4"))
}

fn rotation(label: &str) -> Result<Rotation, String> {
    Rotation::ALL
        .into_iter()
        .find(|rotation| rotation_label(*rotation) == label)
        .ok_or_else(|| format!("giro inválido: {label}"))
}
