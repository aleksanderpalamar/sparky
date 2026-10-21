use std::process::{Command, Stdio};

use zeroize::Zeroizing;

use crate::ports::prompter::{Answer, Ask, Prompter};

pub const PROGRAM: &str = "sparky-keyring-prompt";

pub struct PromptProcess {
    program: String,
}

impl Default for PromptProcess {
    fn default() -> Self {
        Self { program: PROGRAM.to_owned() }
    }
}

pub fn arguments(ask: Ask) -> &'static [&'static str] {
    match ask {
        Ask::Create => &["create"],
        Ask::Unlock => &["unlock"],
        Ask::Retry => &["unlock", "--retry"],
    }
}

pub fn answer(success: bool, output: Vec<u8>) -> Answer {
    let output = Zeroizing::new(output);
    let Ok(text) = std::str::from_utf8(&output) else {
        return Answer::Cancelled;
    };
    match success {
        true => Answer::Password(Zeroizing::new(text.strip_suffix('\n').unwrap_or(text).to_owned())),
        false => Answer::Cancelled,
    }
}

impl Prompter for PromptProcess {
    fn ask(&self, ask: Ask) -> Answer {
        let run = Command::new(&self.program).args(arguments(ask)).stdin(Stdio::null()).stderr(Stdio::inherit()).output();
        match run {
            Ok(output) => answer(output.status.success(), output.stdout),
            Err(error) => {
                eprintln!("sparky-keyring: não deu para abrir {}: {error}", self.program);
                Answer::Cancelled
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{answer, arguments};
    use crate::ports::prompter::{Answer, Ask};

    fn senha(resposta: Answer) -> Option<String> {
        match resposta {
            Answer::Password(senha) => Some(senha.to_string()),
            Answer::Cancelled => None,
        }
    }

    #[test]
    fn cada_pedido_vira_um_modo_da_janela() {
        assert_eq!(arguments(Ask::Create), ["create"]);
        assert_eq!(arguments(Ask::Unlock), ["unlock"]);
        assert_eq!(arguments(Ask::Retry), ["unlock", "--retry"]);
    }

    #[test]
    fn a_senha_vem_pela_saida_sem_a_quebra_final() {
        assert_eq!(senha(answer(true, b"minha senha\n".to_vec())).as_deref(), Some("minha senha"));
        assert_eq!(senha(answer(true, b"com\nquebra\n".to_vec())).as_deref(), Some("com\nquebra"));
    }

    #[test]
    fn janela_fechada_ou_saida_invalida_e_cancelamento() {
        assert_eq!(senha(answer(false, b"x\n".to_vec())), None);
        assert_eq!(senha(answer(true, vec![0xff, 0xfe])), None);
    }
}
