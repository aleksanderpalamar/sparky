use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const KEY_SIZE: usize = 32;
const MOST_MEMORY: u32 = 1 << 20;
const MOST_PASSES: u32 = 16;
const MOST_LANES: u32 = 16;

pub type Key = Zeroizing<[u8; KEY_SIZE]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kdf {
    pub memory: u32,
    pub passes: u32,
    pub lanes: u32,
}

impl Kdf {
    pub const STANDARD: Self = Self { memory: 64 * 1024, passes: 3, lanes: 1 };

    pub fn plausible(&self) -> bool {
        self.memory <= MOST_MEMORY && self.passes <= MOST_PASSES && self.lanes <= MOST_LANES
    }

    pub fn derive(&self, password: &[u8], salt: &[u8]) -> Option<Key> {
        let params = Params::new(self.memory, self.passes, self.lanes, Some(KEY_SIZE)).ok()?;
        let mut key = Zeroizing::new([0u8; KEY_SIZE]);
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
            .hash_password_into(password, salt, &mut key[..])
            .ok()?;
        Some(key)
    }
}

#[cfg(test)]
mod tests {
    use super::Kdf;

    const RAPIDO: Kdf = Kdf { memory: 64, passes: 1, lanes: 1 };

    #[test]
    fn a_mesma_senha_e_o_mesmo_sal_dao_a_mesma_chave() {
        let sal = [7u8; 16];
        assert_eq!(RAPIDO.derive(b"senha", &sal), RAPIDO.derive(b"senha", &sal));
        assert_ne!(RAPIDO.derive(b"senha", &sal), RAPIDO.derive(b"outra", &sal));
        assert_ne!(RAPIDO.derive(b"senha", &sal), RAPIDO.derive(b"senha", &[8u8; 16]));
    }

    #[test]
    fn parametros_absurdos_sao_recusados() {
        assert!(Kdf::STANDARD.plausible());
        assert!(!Kdf { memory: u32::MAX, ..Kdf::STANDARD }.plausible());
        assert_eq!(Kdf { memory: 0, passes: 0, lanes: 0 }.derive(b"x", &[0; 16]), None);
    }
}
