use sparky_config::OutputMode;

pub fn pick(
    available: &[OutputMode],
    preferred: Option<usize>,
    wanted: Option<OutputMode>,
) -> Option<usize> {
    let chosen = wanted.and_then(|wanted| available.iter().position(|mode| wanted.matches(mode)));
    chosen
        .or(preferred)
        .or_else(|| (!available.is_empty()).then_some(0))
}

pub fn kept(names: &[String], disabled: impl Fn(&str) -> bool) -> Vec<bool> {
    let mut kept: Vec<bool> = names.iter().map(|name| !disabled(name)).collect();
    if !kept.contains(&true)
        && let Some(first) = kept.first_mut()
    {
        *first = true;
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::{kept, pick};
    use sparky_config::OutputMode;

    fn modo(width: u32, height: u32, refresh: u32) -> OutputMode {
        OutputMode {
            width,
            height,
            refresh,
        }
    }

    const MODOS: [OutputMode; 3] = [
        OutputMode {
            width: 2560,
            height: 1440,
            refresh: 143_855,
        },
        OutputMode {
            width: 2560,
            height: 1440,
            refresh: 59_951,
        },
        OutputMode {
            width: 1920,
            height: 1080,
            refresh: 60_000,
        },
    ];

    #[test]
    fn o_modo_gravado_vence_o_preferido() {
        assert_eq!(
            pick(&MODOS, Some(0), Some(modo(1920, 1080, 60_000))),
            Some(2)
        );
        assert_eq!(
            pick(&MODOS, Some(0), Some(modo(2560, 1440, 59_951))),
            Some(1)
        );
    }

    #[test]
    fn modo_gravado_que_o_monitor_nao_tem_volta_ao_preferido() {
        assert_eq!(
            pick(&MODOS, Some(1), Some(modo(3840, 2160, 60_000))),
            Some(1)
        );
        assert_eq!(pick(&MODOS, Some(1), None), Some(1));
        assert_eq!(pick(&MODOS, None, None), Some(0));
        assert_eq!(pick(&[], None, Some(modo(1920, 1080, 60_000))), None);
    }

    #[test]
    fn nunca_desliga_todas_as_telas() {
        let nomes = ["DP-1".to_owned(), "HDMI-A-1".to_owned()];
        assert_eq!(kept(&nomes, |name| name == "DP-1"), [false, true]);
        assert_eq!(kept(&nomes, |_| true), [true, false]);
        assert_eq!(kept(&nomes, |_| false), [true, true]);
        assert!(kept(&[], |_| true).is_empty());
    }
}
