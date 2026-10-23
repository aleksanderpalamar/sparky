use std::path::PathBuf;

use crate::domain::launch::{
    Availability, ENVIRONMENT, Launch, LaunchError, Plan, Program, parse, plan,
};

fn ler(argumentos: &[&str]) -> Result<Launch, LaunchError> {
    let argumentos: Vec<String> = argumentos
        .iter()
        .map(|argumento| argumento.to_string())
        .collect();
    parse(&argumentos)
}

fn comando(programa: &str, argumentos: &[&str]) -> Program {
    Program::Command {
        program: programa.to_owned(),
        args: argumentos
            .iter()
            .map(|argumento| argumento.to_string())
            .collect(),
    }
}

#[test]
fn sem_argumentos_abre_o_shell_do_usuario() {
    assert_eq!(
        ler(&[]),
        Ok(Launch {
            program: Program::Shell,
            directory: None
        })
    );
}

#[test]
fn menos_e_roda_o_resto_como_comando_como_o_lancador_pede() {
    assert_eq!(
        ler(&["-e", "htop"]).map(|launch| launch.program),
        Ok(comando("htop", &[]))
    );
    assert_eq!(
        ler(&["-e", "nvim", "-d", "notas.md"]).map(|launch| launch.program),
        Ok(comando("nvim", &["-d", "notas.md"]))
    );
}

#[test]
fn a_pasta_inicial_vem_de_d_ou_working_directory() {
    let pasta = Some(PathBuf::from("/tmp"));
    assert_eq!(
        ler(&["-d", "/tmp"]).map(|launch| launch.directory),
        Ok(pasta.clone())
    );
    assert_eq!(
        ler(&["--working-directory", "/tmp"]).map(|launch| launch.directory),
        Ok(pasta.clone())
    );
    assert_eq!(
        ler(&["-d", "/tmp", "-e", "ls", "-la"]),
        Ok(Launch {
            program: comando("ls", &["-la"]),
            directory: pasta
        })
    );
}

#[test]
fn opcoes_incompletas_ou_desconhecidas_sao_recusadas() {
    assert_eq!(ler(&["-e"]), Err(LaunchError::MissingCommand));
    assert_eq!(ler(&["-d"]), Err(LaunchError::MissingDirectory));
    assert_eq!(
        ler(&["--voar"]),
        Err(LaunchError::Unknown("--voar".to_owned()))
    );
}

#[test]
fn o_ambiente_anuncia_cores_para_o_powerlevel10k() {
    assert!(ENVIRONMENT.contains(&("TERM", "xterm-256color")));
    assert!(ENVIRONMENT.contains(&("COLORTERM", "truecolor")));
    assert!(ENVIRONMENT.contains(&("TERM_PROGRAM", "sparky-terminal")));
}

fn ambiente(plano: &Plan, nome: &str) -> Option<String> {
    plano
        .environment
        .iter()
        .find(|(chave, _)| chave == nome)
        .map(|(_, valor)| valor.clone())
}

#[test]
fn o_shell_padrao_e_o_zsh_com_a_configuracao_do_sparky() {
    let shell = Launch {
        program: Program::Shell,
        directory: None,
    };
    let plano = plan(
        &shell,
        Availability::Installed,
        std::path::Path::new("/usr/share/sparky/zsh"),
    );
    assert_eq!(plano.program, Some(("zsh".to_owned(), Vec::new())));
    assert_eq!(
        ambiente(&plano, "ZDOTDIR").as_deref(),
        Some("/usr/share/sparky/zsh")
    );
    assert_eq!(ambiente(&plano, "TERM").as_deref(), Some("xterm-256color"));
}

#[test]
fn sem_zsh_instalado_cai_no_shell_da_conta() {
    let shell = Launch {
        program: Program::Shell,
        directory: None,
    };
    assert_eq!(
        plan(&shell, Availability::Missing, std::path::Path::new("/z")).program,
        None
    );
}

#[test]
fn um_comando_roda_como_pedido() {
    let comando = Launch {
        program: Program::Command {
            program: "htop".to_owned(),
            args: vec!["-d".to_owned()],
        },
        directory: None,
    };
    let plano = plan(
        &comando,
        Availability::Installed,
        std::path::Path::new("/z"),
    );
    assert_eq!(
        plano.program,
        Some(("htop".to_owned(), vec!["-d".to_owned()]))
    );
}
