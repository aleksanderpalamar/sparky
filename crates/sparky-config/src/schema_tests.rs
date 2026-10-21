use sparky_tiling::{Direction, Fullscreen, Gaps, Style};

use crate::action::Action;
use crate::load::{ConfigError, parse};
use crate::schema::{Config, General, Input};

#[test]
fn arquivo_vazio_vale_os_padroes_do_sparky() {
    assert_eq!(parse(""), Ok(Config::default()));
    assert_eq!(Config::default().general, General::SPARKY);
    assert_eq!(Config::default().general.style(), Style::SPARKY);
}

#[test]
fn o_que_nao_foi_escrito_continua_no_padrao() {
    let Ok(config) = parse("[general]\ngaps_outer = 20\n") else {
        panic!("deveria ler");
    };
    assert_eq!(config.general.gaps_outer, 20);
    assert_eq!(config.general.gaps_inner, 5);
    assert_eq!(config.general.style().gaps, Gaps { inner: 5, outer: 20 });
}

#[test]
fn le_o_teclado_e_a_repeticao() {
    let Ok(config) = parse("[input]\nlayout = \"br\"\nvariant = \"abnt2\"\nrepeat_rate = 30\n") else {
        panic!("deveria ler");
    };
    assert_eq!(config.input.layout.as_deref(), Some("br"));
    assert_eq!(config.input.variant.as_deref(), Some("abnt2"));
    assert_eq!(config.input.repeat_rate, 30);
    assert_eq!(config.input.repeat_delay, Input::SPARKY.repeat_delay);
}

#[test]
fn le_os_atalhos_com_a_acao_e_os_campos_dela() {
    let contents = r#"
[[bind]]
keys = "SUPER + B"
action = "spawn"
command = "firefox"

[[bind]]
keys = "SUPER + H"
action = "focus"
direction = "left"

[[bind]]
keys = "SUPER + M"
action = "fullscreen"
mode = "maximized"

[[bind]]
keys = "SUPER + SHIFT + code:12"
action = "move-to-workspace"
workspace = 3
"#;
    let Ok(config) = parse(contents) else {
        panic!("deveria ler");
    };
    let actions: Vec<Action> = config.bind.into_iter().map(|bind| bind.action).collect();
    assert_eq!(
        actions,
        vec![
            Action::Spawn { command: "firefox".to_owned() },
            Action::Focus { direction: Direction::Left },
            Action::Fullscreen { mode: Fullscreen::Maximized },
            Action::MoveToWorkspace { workspace: 3, follow: true },
        ]
    );
}

#[test]
fn le_o_autostart_extra() {
    let Ok(config) = parse("autostart = [\"nm-applet\", \"blueman-applet\"]\n") else {
        panic!("deveria ler");
    };
    assert_eq!(config.autostart, vec!["nm-applet", "blueman-applet"]);
}

#[test]
fn chave_desconhecida_e_erro_para_nao_esconder_erro_de_digitacao() {
    assert!(matches!(parse("[general]\ngap_outer = 20\n"), Err(ConfigError::Invalid(_))));
}

#[test]
fn acao_desconhecida_e_erro() {
    let contents = "[[bind]]\nkeys = \"SUPER + X\"\naction = \"voar\"\n";
    assert!(matches!(parse(contents), Err(ConfigError::Invalid(_))));
}

#[test]
fn toml_quebrado_e_erro_com_a_mensagem() {
    let Err(ConfigError::Invalid(message)) = parse("[general\n") else {
        panic!("deveria recusar");
    };
    assert!(!message.is_empty());
}

#[test]
fn le_as_acoes_do_rascunho_pelo_nome() {
    let contents = "[[bind]]\nkeys = \"SUPER + Z\"\naction = \"toggle-scratchpad\"\n\n[[bind]]\nkeys = \"SUPER + X\"\naction = \"move-to-scratchpad\"\n";
    let Ok(config) = parse(contents) else {
        panic!("as ações do rascunho deveriam ser aceitas");
    };
    let acoes: Vec<Action> = config.bind.into_iter().map(|bind| bind.action).collect();
    assert_eq!(acoes, [Action::ToggleScratchpad, Action::MoveToScratchpad]);
}

#[test]
fn le_as_acoes_de_grupo_com_os_campos() {
    let contents = "[[bind]]\nkeys = \"SUPER + Z\"\naction = \"move-into-group\"\ndirection = \"left\"\n\n[[bind]]\nkeys = \"SUPER + X\"\naction = \"select-in-group\"\nindex = 2\n";
    let Ok(config) = parse(contents) else {
        panic!("as ações de grupo deveriam ser aceitas");
    };
    let acoes: Vec<Action> = config.bind.into_iter().map(|bind| bind.action).collect();
    assert_eq!(acoes, [Action::MoveIntoGroup { direction: Direction::Left }, Action::SelectInGroup { index: 2 }]);
}

#[test]
fn o_lancador_padrao_e_o_do_sparky() {
    assert_eq!(Config::default().apps.launcher, "sparky-launcher");
}

#[test]
fn o_locker_padrao_e_o_swaylock_e_pode_trocar() {
    assert_eq!(Config::default().apps.locker, "swaylock");
    let Ok(config) = parse("[apps]\nlocker = \"hyprlock\"\n") else { panic!("deveria ler o locker") };
    assert_eq!(config.apps.locker, "hyprlock");
}

#[test]
fn o_papel_de_parede_e_opcional_no_general() {
    assert_eq!(Config::default().general.wallpaper, None);
    let Ok(config) = parse("[general]\nwallpaper = \"~/Imagens/fundo.webp\"\n") else {
        panic!("o sparky.toml com papel de parede deveria ser aceito");
    };
    assert_eq!(config.general.wallpaper.as_deref(), Some("~/Imagens/fundo.webp"));
}
