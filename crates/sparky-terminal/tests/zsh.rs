use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn configuracao() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/zsh")
}

fn zsh(casa: &Path, pasta: &Path, script: &str) -> Output {
    let rodou = Command::new("zsh")
        .args(["-i", "-c", script])
        .current_dir(pasta)
        .env_clear()
        .env("HOME", casa)
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("TERM", "xterm-256color")
        .env("COLUMNS", "100")
        .env("ZDOTDIR", configuracao())
        .env("XDG_STATE_HOME", casa.join("estado"))
        .env("XDG_CACHE_HOME", casa.join("cache"))
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output();
    match rodou {
        Ok(saida) => saida,
        Err(erro) => {
            panic!("o zsh precisa estar instalado para testar a configuração do sparky: {erro}")
        }
    }
}

fn texto(saida: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    )
}

fn casa_com_armadilha() -> tempfile::TempDir {
    let Ok(casa) = tempfile::tempdir() else {
        panic!("sem pasta temporária")
    };
    let armadilhas = [".zshrc", ".zshenv", ".zprofile", ".p10k.zsh"];
    for nome in armadilhas {
        assert!(std::fs::write(casa.path().join(nome), "print LEU_O_HOME\n").is_ok());
    }
    casa
}

#[test]
fn os_arquivos_da_configuracao_sao_zsh_valido() {
    let Ok(arquivos) = std::fs::read_dir(configuracao()) else {
        panic!("sem data/zsh")
    };
    for arquivo in arquivos.flatten() {
        let validou = Command::new("zsh").arg("-n").arg(arquivo.path()).output();
        assert!(
            validou.is_ok_and(|saida| saida.status.success()),
            "{} não passou no zsh -n",
            arquivo.path().display()
        );
    }
}

#[test]
fn nao_le_nada_do_home_nem_oh_my_zsh_nem_p10k() {
    let casa = casa_com_armadilha();
    let saida = zsh(casa.path(), casa.path(), "print -r -- PRONTO");
    let lido = texto(&saida);
    assert!(lido.contains("PRONTO"), "{lido}");
    assert!(!lido.contains("LEU_O_HOME"), "{lido}");
    for proibido in [".oh-my-zsh", "p10k", "powerlevel"] {
        assert!(!lido.contains(proibido), "{proibido} em {lido}");
    }
}

#[test]
fn o_prompt_tem_a_moldura_o_pontilhado_e_o_status() {
    let casa = casa_com_armadilha();
    let saida = zsh(
        casa.path(),
        casa.path(),
        "sparky-precmd; print -r -- \"$PROMPT\"",
    );
    let prompt = texto(&saida);
    for parte in ["╭─", "╰─", "·", "✔", "%K{4}"] {
        assert!(prompt.contains(parte), "{parte} faltando em {prompt}");
    }
}

#[test]
fn o_prompt_mostra_o_ramo_e_as_mudancas_do_git() {
    let casa = casa_com_armadilha();
    let projeto = casa.path().join("projeto");
    assert!(std::fs::create_dir(&projeto).is_ok());
    let preparo = "git init -q -b principal && git -c user.name=t -c user.email=t@t commit -q --allow-empty -m a && echo x > novo";
    assert!(
        Command::new("sh")
            .args(["-c", preparo])
            .current_dir(&projeto)
            .env("HOME", casa.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .status()
            .is_ok_and(|status| status.success())
    );
    let saida = zsh(
        casa.path(),
        &projeto,
        "sparky-precmd; print -r -- \"$PROMPT\"",
    );
    let prompt = texto(&saida);
    assert!(prompt.contains("principal"), "{prompt}");
    assert!(prompt.contains("?1"), "{prompt}");
}

#[test]
fn o_historico_fica_na_pasta_de_estado_do_sparky() {
    let casa = casa_com_armadilha();
    let saida = zsh(casa.path(), casa.path(), "print -r -- $HISTFILE");
    assert!(
        texto(&saida).contains(
            &casa
                .path()
                .join("estado/sparky/zsh_history")
                .display()
                .to_string()
        ),
        "{}",
        texto(&saida)
    );
}

#[test]
fn a_busca_na_web_codifica_acentos_e_espacos() {
    let casa = casa_com_armadilha();
    let saida = zsh(casa.path(), casa.path(), "sparky-uri 'olá mundo & café'");
    assert_eq!(
        String::from_utf8_lossy(&saida.stdout),
        "ol%C3%A1%20mundo%20%26%20caf%C3%A9"
    );
}
