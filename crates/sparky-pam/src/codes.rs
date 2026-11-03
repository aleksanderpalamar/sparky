use std::ffi::c_int;

use sparky_login_core::protocol::Refusal;

pub const SUCCESS: c_int = 0;
pub const BUF_ERR: c_int = 5;
pub const PERM_DENIED: c_int = 6;
pub const AUTH_ERR: c_int = 7;
pub const CRED_INSUFFICIENT: c_int = 8;
pub const USER_UNKNOWN: c_int = 10;
pub const MAXTRIES: c_int = 11;
pub const NEW_AUTHTOK_REQD: c_int = 12;
pub const ACCT_EXPIRED: c_int = 13;
pub const CONV_ERR: c_int = 19;

pub const TTY: c_int = 3;
pub const ESTABLISH_CRED: c_int = 0x0002;
pub const DELETE_CRED: c_int = 0x0004;

pub const PROMPT_ECHO_OFF: c_int = 1;
pub const PROMPT_ECHO_ON: c_int = 2;
pub const ERROR_MSG: c_int = 3;
pub const TEXT_INFO: c_int = 4;

const EXPIRED_PASSWORD: &str = "a senha expirou; troque-a num terminal com passwd";
const EXPIRED_ACCOUNT: &str = "a conta expirou";

pub fn refusal(code: c_int, notes: &[String], explanation: impl FnOnce() -> String) -> Refusal {
    match code {
        NEW_AUTHTOK_REQD => Refusal::Unavailable(EXPIRED_PASSWORD.to_owned()),
        ACCT_EXPIRED => Refusal::Unavailable(EXPIRED_ACCOUNT.to_owned()),
        _ if !notes.is_empty() => Refusal::Unavailable(notes.join(" ")),
        AUTH_ERR | USER_UNKNOWN | MAXTRIES | PERM_DENIED | CRED_INSUFFICIENT => Refusal::Denied,
        _ => Refusal::Unavailable(explanation()),
    }
}

#[cfg(test)]
mod tests {
    use sparky_login_core::protocol::Refusal;

    use super::{ACCT_EXPIRED, AUTH_ERR, MAXTRIES, NEW_AUTHTOK_REQD, USER_UNKNOWN, refusal};

    fn sem_texto() -> String {
        "erro do PAM".to_owned()
    }

    #[test]
    fn senha_errada_e_usuario_desconhecido_dao_a_mesma_recusa() {
        for code in [AUTH_ERR, USER_UNKNOWN, MAXTRIES] {
            assert_eq!(refusal(code, &[], sem_texto), Refusal::Denied);
        }
    }

    #[test]
    fn senha_e_conta_expiradas_explicam_o_motivo() {
        assert_eq!(
            refusal(NEW_AUTHTOK_REQD, &[], sem_texto),
            Refusal::Unavailable("a senha expirou; troque-a num terminal com passwd".into())
        );
        assert_eq!(
            refusal(ACCT_EXPIRED, &[], sem_texto),
            Refusal::Unavailable("a conta expirou".into())
        );
    }

    #[test]
    fn outros_erros_usam_a_explicacao_do_pam() {
        assert_eq!(
            refusal(9, &[], sem_texto),
            Refusal::Unavailable("erro do PAM".into())
        );
    }

    #[test]
    fn as_mensagens_do_pam_explicam_a_recusa() {
        let notas = [
            "The account is locked due to 3 failed logins.".to_owned(),
            "(10 minutes left to unlock)".to_owned(),
        ];
        assert_eq!(
            refusal(AUTH_ERR, &notas, sem_texto),
            Refusal::Unavailable(
                "The account is locked due to 3 failed logins. (10 minutes left to unlock)".into()
            )
        );
    }

    #[test]
    fn senha_expirada_continua_com_a_explicacao_do_sparky() {
        let notas = ["You are required to change your password immediately".to_owned()];
        assert_eq!(
            refusal(NEW_AUTHTOK_REQD, &notas, sem_texto),
            Refusal::Unavailable("a senha expirou; troque-a num terminal com passwd".into())
        );
    }
}
