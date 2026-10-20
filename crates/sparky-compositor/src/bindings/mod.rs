use smithay::input::keyboard::{Keysym, ModifiersState, xkb};
use sparky_config::combo::{Combo, KeySpec, Mods};
use sparky_config::{Action, Bind};

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
    pub fn new(binds: Vec<Bind>) -> Self {
        let entries = binds
            .into_iter()
            .filter_map(|bind| binding(&bind.keys, bind.action))
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
    use smithay::input::keyboard::{Keysym, xkb};
    use sparky_config::combo::Mods;
    use sparky_config::{Action, Bind};

    fn abrir(command: &str) -> Action {
        Action::Spawn {
            command: command.to_owned(),
        }
    }

    fn tabela() -> Bindings {
        Bindings::new(vec![
            Bind::new("SUPER + RETURN", abrir("kitty")),
            Bind::new("SUPER + SHIFT + F", abrir("sparky-files")),
            Bind::new("SUPER + code:10", Action::Workspace { workspace: 1 }),
            Bind::new("SUPER + TECLA_QUE_NAO_EXISTE", Action::Exit),
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
        assert_eq!(tabela().find(super_(), 36, sym("Return")), Some(&abrir("kitty")));
    }

    #[test]
    fn letra_maiuscula_no_atalho_casa_com_a_tecla_sem_shift_aplicado() {
        let mods = Mods {
            shift: true,
            ..super_()
        };
        assert_eq!(
            tabela().find(mods, 41, sym("f")),
            Some(&abrir("sparky-files"))
        );
    }

    #[test]
    fn atalho_por_codigo_ignora_o_simbolo() {
        assert_eq!(tabela().find(super_(), 10, sym("exclam")), Some(&Action::Workspace { workspace: 1 }));
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
