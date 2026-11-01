use serde::{Deserialize, Serialize};
use sparky_config::{OutputMode, OutputScale, Rotation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vrr {
    #[default]
    Unsupported,
    Off,
    On,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OutputInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub focused: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub mode: Option<OutputMode>,
    #[serde(default)]
    pub modes: Vec<OutputMode>,
    #[serde(default)]
    pub scale: OutputScale,
    #[serde(default)]
    pub rotation: Rotation,
    #[serde(default)]
    pub vrr: Vrr,
    #[serde(default = "enabled")]
    pub enabled: bool,
}

const fn enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{OutputInfo, Vrr};
    use sparky_config::{OutputMode, OutputScale, Rotation};

    #[test]
    fn resposta_antiga_sem_os_detalhes_continua_valida() {
        let json = r#"{"name":"DP-1","x":0,"y":0,"width":1920,"height":1080,"focused":true}"#;
        let Ok(output) = serde_json::from_str::<OutputInfo>(json) else {
            panic!("deveria ler")
        };
        assert!(output.enabled);
        assert_eq!(output.scale, OutputScale::ONE);
        assert_eq!(output.rotation, Rotation::Normal);
        assert_eq!(output.vrr, Vrr::Unsupported);
        assert!(output.modes.is_empty());
    }

    #[test]
    fn os_detalhes_do_monitor_vao_e_voltam() {
        let modo = OutputMode {
            width: 2560,
            height: 1440,
            refresh: 143_855,
        };
        let output = OutputInfo {
            name: "DP-1".to_owned(),
            description: "Dell U2723QE".to_owned(),
            mode: Some(modo),
            modes: vec![
                modo,
                OutputMode {
                    width: 1920,
                    height: 1080,
                    refresh: 60_000,
                },
            ],
            scale: OutputScale::from_factor(1.25).unwrap_or_default(),
            rotation: Rotation::Left,
            vrr: Vrr::On,
            enabled: false,
            ..OutputInfo::default()
        };
        let json = serde_json::to_string(&output).unwrap_or_default();
        assert!(json.contains(r#""mode":"2560x1440@143.855""#), "{json}");
        assert!(json.contains(r#""rotation":270"#), "{json}");
        assert_eq!(serde_json::from_str::<OutputInfo>(&json).ok(), Some(output));
    }
}
