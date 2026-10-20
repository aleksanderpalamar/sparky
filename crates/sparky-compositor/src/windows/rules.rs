use sparky_tiling::{Opening, Style};

const FLOATING_APPS: [&str; 6] = [
    "sparky-settings",
    "sparky-shot",
    "org.pulseaudio.pavucontrol",
    "nm-connection-editor",
    "blueman-manager",
    "xdg-desktop-portal-gtk",
];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Traits {
    pub app_id: String,
    pub has_parent: bool,
    pub min_size: (i32, i32),
    pub max_size: (i32, i32),
}

pub fn opening(traits: &Traits, style: Style, border: i32) -> Opening {
    if fixed_size(traits) {
        let (width, height) = traits.min_size;
        return Opening::Floating(width + 2 * border, height + 2 * border);
    }
    let floating = traits.has_parent || FLOATING_APPS.contains(&traits.app_id.as_str());
    match floating {
        true => Opening::Floating(style.float_size.0, style.float_size.1),
        false => Opening::Tiled,
    }
}

fn fixed_size(traits: &Traits) -> bool {
    let (min_width, min_height) = traits.min_size;
    min_width > 0 && min_height > 0 && traits.min_size == traits.max_size
}

#[cfg(test)]
mod tests {
    use super::{Traits, opening};
    use sparky_tiling::{Opening, Style};

    fn janela(app_id: &str) -> Traits {
        Traits {
            app_id: app_id.to_owned(),
            ..Traits::default()
        }
    }

    #[test]
    fn janela_comum_entra_no_mosaico() {
        assert_eq!(opening(&janela("kitty"), Style::SPARKY, 2), Opening::Tiled);
    }

    #[test]
    fn os_aplicativos_de_dialogo_do_sparky_flutuam() {
        assert_eq!(opening(&janela("sparky-settings"), Style::SPARKY, 2), Opening::Floating(875, 600));
    }

    #[test]
    fn janela_com_mae_e_dialogo_e_flutua() {
        let dialogo = Traits {
            has_parent: true,
            ..janela("org.gnome.Nautilus")
        };
        assert_eq!(opening(&dialogo, Style::SPARKY, 2), Opening::Floating(875, 600));
    }

    #[test]
    fn tamanho_fixo_flutua_no_proprio_tamanho_com_a_borda() {
        let fixa = Traits {
            min_size: (400, 300),
            max_size: (400, 300),
            ..janela("calculadora")
        };
        assert_eq!(opening(&fixa, Style::SPARKY, 2), Opening::Floating(404, 304));
    }

    #[test]
    fn so_tamanho_minimo_nao_e_tamanho_fixo() {
        let minima = Traits {
            min_size: (400, 300),
            ..janela("editor")
        };
        assert_eq!(opening(&minima, Style::SPARKY, 2), Opening::Tiled);
    }
}
