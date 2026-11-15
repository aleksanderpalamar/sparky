use crate::renamed::wallpaper;
use crate::{migrate, parse};

#[test]
fn os_papeis_de_parede_dos_temas_renomeados_vao_para_a_pasta_nova() {
    assert_eq!(
        wallpaper("/usr/share/sparky/wallpapers/spark/1-aurora.webp".to_owned()),
        "/usr/share/sparky/wallpapers/sparky/1-aurora.webp"
    );
    assert_eq!(
        wallpaper("/usr/share/sparky/wallpapers/spark-98/2-teal-dither.webp".to_owned()),
        "/usr/share/sparky/wallpapers/sparky-98/2-teal-dither.webp"
    );
    assert_eq!(
        wallpaper("~/.local/share/sparky/wallpapers/spark/minha.webp".to_owned()),
        "~/.local/share/sparky/wallpapers/sparky/minha.webp"
    );
}

#[test]
fn outros_caminhos_ficam_como_estao() {
    for caminho in [
        "/usr/share/sparky/wallpapers/sparky/1-aurora.webp",
        "/usr/share/sparky/wallpapers/sparky-98/1-teal.webp",
        "/usr/share/sparky/wallpapers/tokyo-night/1-lago.webp",
        "/home/ana/Imagens/spark/fundo.webp",
        "",
    ] {
        assert_eq!(wallpaper(caminho.to_owned()), caminho);
    }
}

#[test]
fn a_configuracao_antiga_ja_chega_corrigida_para_quem_a_le() {
    let config = parse(
        "[general]\nwallpaper = \"/usr/share/sparky/wallpapers/spark/1-aurora.webp\"\n\n[login]\nwallpaper = \"/usr/share/sparky/wallpapers/spark-98/1-teal.webp\"\n",
    );
    let Ok(config) = config else {
        panic!("configuração válida");
    };
    assert_eq!(
        config.general.wallpaper.as_deref(),
        Some("/usr/share/sparky/wallpapers/sparky/1-aurora.webp")
    );
    assert_eq!(
        config.login.wallpaper.as_deref(),
        Some("/usr/share/sparky/wallpapers/sparky-98/1-teal.webp")
    );
}

#[test]
fn migrar_regrava_so_os_caminhos_antigos_e_preserva_o_resto() {
    let antigo = "# meu sparky\n[general]\nborder = 3\nwallpaper = \"/usr/share/sparky/wallpapers/spark/1-aurora.webp\" # fundo\n\n[login]\nwallpaper = \"/usr/share/sparky/wallpapers/spark-98/1-teal.webp\"\n";
    let novo = "# meu sparky\n[general]\nborder = 3\nwallpaper = \"/usr/share/sparky/wallpapers/sparky/1-aurora.webp\" # fundo\n\n[login]\nwallpaper = \"/usr/share/sparky/wallpapers/sparky-98/1-teal.webp\"\n";
    assert_eq!(migrate(antigo).as_deref(), Some(novo));
}

#[test]
fn sem_caminho_antigo_ou_com_arquivo_invalido_nada_e_regravado() {
    assert_eq!(migrate("[general]\nborder = 3\n"), None);
    assert_eq!(
        migrate("[general]\nwallpaper = \"/usr/share/sparky/wallpapers/sparky/1-aurora.webp\"\n"),
        None
    );
    assert_eq!(migrate("[general"), None);
}
