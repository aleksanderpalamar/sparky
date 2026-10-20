use smithay::input::keyboard::XkbConfig;
use sparky_config::Input;

use crate::input::keyboard_layout::KeyboardLayout;

pub fn layout(input: &Input, system: KeyboardLayout) -> KeyboardLayout {
    let Some(chosen) = input.layout.as_deref().filter(|layout| !layout.trim().is_empty()) else {
        return system;
    };
    let text = |value: &Option<String>| value.clone().unwrap_or_default();
    KeyboardLayout {
        layout: chosen.to_owned(),
        variant: text(&input.variant),
        model: text(&input.model),
        options: text(&input.options),
    }
}

pub fn xkb_config(layout: &KeyboardLayout) -> XkbConfig<'_> {
    XkbConfig {
        layout: &layout.layout,
        variant: &layout.variant,
        model: &layout.model,
        options: Some(layout.options.clone()).filter(|options| !options.is_empty()),
        ..XkbConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::layout;
    use crate::input::keyboard_layout::KeyboardLayout;
    use sparky_config::Input;

    fn sistema() -> KeyboardLayout {
        KeyboardLayout {
            layout: "br".to_owned(),
            variant: "abnt2".to_owned(),
            model: String::new(),
            options: String::new(),
        }
    }

    #[test]
    fn sem_layout_na_configuracao_vale_o_do_sistema() {
        assert_eq!(layout(&Input::SPARKY, sistema()), sistema());
    }

    #[test]
    fn o_layout_da_configuracao_vence_o_do_sistema_inteiro() {
        let input = Input {
            layout: Some("us".to_owned()),
            options: Some("ctrl:nocaps".to_owned()),
            ..Input::SPARKY
        };
        let chosen = layout(&input, sistema());
        assert_eq!(chosen.layout, "us");
        assert_eq!(chosen.variant, "");
        assert_eq!(chosen.options, "ctrl:nocaps");
    }
}
