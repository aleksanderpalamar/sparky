pub mod action;
pub mod combo;
pub mod defaults;

use smithay::input::keyboard::{Keysym, ModifiersState, xkb};

use crate::bindings::action::Action;
use crate::bindings::combo::{Combo, KeySpec, Mods};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    Code(u32),
    Sym(Keysym),
}

#[derive(Debug, Clone)]
struct Binding {
    mods: Mods,
    key: Key,
    action: Action,
}

#[derive(Debug, Clone, Default)]
pub struct Bindings {
    entries: Vec<Binding>,
}

impl Bindings {
    pub fn new(specs: Vec<(String, Action)>) -> Self {
        let entries = specs
            .into_iter()
            .filter_map(|(keys, action)| binding(&keys, action))
            .collect();
        Self { entries }
    }

    pub fn find(&self, mods: Mods, code: u32, sym: Option<Keysym>) -> Option<&Action> {
        self.entries
            .iter()
            .find(|binding| binding.mods == mods && matches(binding.key, code, sym))
            .map(|binding| &binding.action)
    }
}

pub fn mods_of(state: &ModifiersState) -> Mods {
    Mods {
        logo: state.logo,
        shift: state.shift,
        ctrl: state.ctrl,
        alt: state.alt,
    }
}

fn matches(key: Key, code: u32, sym: Option<Keysym>) -> bool {
    match key {
        Key::Code(expected) => expected == code,
        Key::Sym(expected) => sym == Some(expected),
    }
}

fn binding(keys: &str, action: Action) -> Option<Binding> {
    let combo = match Combo::parse(keys) {
        Ok(combo) => combo,
        Err(error) => {
            tracing::warn!("atalho inválido {keys:?}: {error:?}");
            return None;
        }
    };
    let key = resolve(&combo.key)?;
    Some(Binding {
        mods: combo.mods,
        key,
        action,
    })
}

fn resolve(spec: &KeySpec) -> Option<Key> {
    match spec {
        KeySpec::Code(code) => Some(Key::Code(*code)),
        KeySpec::Name(name) => {
            let sym = xkb::keysym_from_name(name, xkb::KEYSYM_CASE_INSENSITIVE);
            match sym.raw() == xkb::keysyms::KEY_NoSymbol {
                true => {
                    tracing::warn!("tecla desconhecida no atalho: {name}");
                    None
                }
                false => Some(Key::Sym(sym)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bindings;
    use crate::bindings::action::Action;
    use crate::bindings::combo::Mods;
    use smithay::input::keyboard::{Keysym, xkb};

    fn tabela() -> Bindings {
        Bindings::new(vec![
            ("SUPER + RETURN".to_owned(), Action::Spawn("kitty".to_owned())),
            ("SUPER + SHIFT + F".to_owned(), Action::Spawn("sparky-files".to_owned())),
            ("SUPER + code:10".to_owned(), Action::Workspace(1)),
            ("SUPER + TECLA_QUE_NAO_EXISTE".to_owned(), Action::Exit),
        ])
    }

    fn super_() -> Mods {
        Mods {
            logo: true,
            ..Mods::default()
        }
    }

    fn sym(name: &str) -> Option<Keysym> {
        Some(xkb::keysym_from_name(name, xkb::KEYSYM_NO_FLAGS))
    }

    #[test]
    fn acha_pela_tecla_e_pelos_modificadores() {
        assert_eq!(tabela().find(super_(), 36, sym("Return")), Some(&Action::Spawn("kitty".to_owned())));
    }

    #[test]
    fn letra_maiuscula_no_atalho_casa_com_a_tecla_sem_shift_aplicado() {
        let mods = Mods {
            shift: true,
            ..super_()
        };
        assert_eq!(
            tabela().find(mods, 41, sym("f")),
            Some(&Action::Spawn("sparky-files".to_owned()))
        );
    }

    #[test]
    fn atalho_por_codigo_ignora_o_simbolo() {
        assert_eq!(tabela().find(super_(), 10, sym("exclam")), Some(&Action::Workspace(1)));
    }

    #[test]
    fn modificador_a_mais_nao_casa() {
        let mods = Mods {
            ctrl: true,
            ..super_()
        };
        assert_eq!(tabela().find(mods, 36, sym("Return")), None);
    }

    #[test]
    fn atalho_com_tecla_desconhecida_fica_de_fora() {
        assert_eq!(tabela().entries.len(), 3);
    }
}
