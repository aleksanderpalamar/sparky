use crate::dwindle::Dwindle;
use crate::geometry::Rect;
use crate::ids::WindowId;

const TELA: Rect = Rect::new(0, 0, 1920, 1080);

fn janela(id: u64) -> WindowId {
    WindowId(id)
}

fn arvore(ids: &[u64]) -> Dwindle {
    let mut dwindle = Dwindle::default();
    let mut foco = None;
    for id in ids {
        dwindle.insert(janela(*id), foco, TELA);
        foco = Some(janela(*id));
    }
    dwindle
}

fn caixa(dwindle: &Dwindle, id: u64) -> Rect {
    let found = dwindle.layout(TELA).into_iter().find(|(window, _)| *window == janela(id));
    let Some((_, rect)) = found else {
        panic!("janela {id} fora do layout");
    };
    rect
}

#[test]
fn a_primeira_janela_ocupa_a_tela_toda() {
    assert_eq!(caixa(&arvore(&[1]), 1), TELA);
}

#[test]
fn a_segunda_janela_vai_para_a_direita_na_tela_larga() {
    let dwindle = arvore(&[1, 2]);
    assert_eq!(caixa(&dwindle, 1), Rect::new(0, 0, 960, 1080));
    assert_eq!(caixa(&dwindle, 2), Rect::new(960, 0, 960, 1080));
}

#[test]
fn a_terceira_divide_a_janela_em_foco_empilhando() {
    let dwindle = arvore(&[1, 2, 3]);
    assert_eq!(caixa(&dwindle, 1), Rect::new(0, 0, 960, 1080));
    assert_eq!(caixa(&dwindle, 2), Rect::new(960, 0, 960, 540));
    assert_eq!(caixa(&dwindle, 3), Rect::new(960, 540, 960, 540));
}

#[test]
fn a_quarta_espirala_para_dentro_como_no_hyprland() {
    let dwindle = arvore(&[1, 2, 3, 4]);
    assert_eq!(caixa(&dwindle, 3), Rect::new(960, 540, 480, 540));
    assert_eq!(caixa(&dwindle, 4), Rect::new(1440, 540, 480, 540));
}

#[test]
fn a_janela_nova_divide_a_janela_alvo_e_nao_a_ultima() {
    let mut dwindle = arvore(&[1, 2]);
    dwindle.insert(janela(3), Some(janela(1)), TELA);
    assert_eq!(caixa(&dwindle, 1), Rect::new(0, 0, 960, 540));
    assert_eq!(caixa(&dwindle, 3), Rect::new(0, 540, 960, 540));
    assert_eq!(caixa(&dwindle, 2), Rect::new(960, 0, 960, 1080));
}

#[test]
fn alvo_desconhecido_divide_a_ultima_janela() {
    let mut dwindle = arvore(&[1, 2]);
    dwindle.insert(janela(3), Some(janela(99)), TELA);
    assert_eq!(caixa(&dwindle, 3), Rect::new(960, 540, 960, 540));
}

#[test]
fn inserir_de_novo_nao_duplica() {
    let mut dwindle = arvore(&[1, 2]);
    dwindle.insert(janela(2), None, TELA);
    assert_eq!(dwindle.windows(), vec![janela(1), janela(2)]);
}

#[test]
fn fechar_uma_janela_devolve_o_espaco_para_a_vizinha() {
    let mut dwindle = arvore(&[1, 2, 3]);
    assert!(dwindle.remove(janela(2)));
    assert_eq!(caixa(&dwindle, 3), Rect::new(960, 0, 960, 1080));
    assert_eq!(caixa(&dwindle, 1), Rect::new(0, 0, 960, 1080));
}

#[test]
fn fechar_a_ultima_esvazia_a_arvore() {
    let mut dwindle = arvore(&[1]);
    assert!(dwindle.remove(janela(1)));
    assert!(dwindle.is_empty());
}

#[test]
fn fechar_janela_que_nao_existe_nao_muda_nada() {
    let mut dwindle = arvore(&[1, 2]);
    assert!(!dwindle.remove(janela(9)));
    assert_eq!(dwindle.windows().len(), 2);
}

#[test]
fn alternar_a_divisao_empilha_o_par() {
    let mut dwindle = arvore(&[1, 2]);
    assert!(dwindle.toggle_split(janela(2)));
    assert_eq!(caixa(&dwindle, 1), Rect::new(0, 0, 1920, 540));
    assert_eq!(caixa(&dwindle, 2), Rect::new(0, 540, 1920, 540));
}

#[test]
fn janela_sozinha_nao_tem_divisao_para_alternar() {
    let mut dwindle = arvore(&[1]);
    assert!(!dwindle.toggle_split(janela(1)));
}

#[test]
fn trocar_duas_janelas_troca_as_caixas() {
    let mut dwindle = arvore(&[1, 2, 3]);
    assert!(dwindle.swap(janela(1), janela(3)));
    assert_eq!(caixa(&dwindle, 3), Rect::new(0, 0, 960, 1080));
    assert_eq!(caixa(&dwindle, 1), Rect::new(960, 540, 960, 540));
}

#[test]
fn trocar_com_janela_de_fora_nao_faz_nada() {
    let mut dwindle = arvore(&[1, 2]);
    assert!(!dwindle.swap(janela(1), janela(7)));
    assert_eq!(caixa(&dwindle, 1), Rect::new(0, 0, 960, 1080));
}

#[test]
fn aumentar_a_largura_move_a_divisao_lado_a_lado() {
    let mut dwindle = arvore(&[1, 2]);
    dwindle.resize(janela(1), 192, 0, TELA);
    assert_eq!(caixa(&dwindle, 1).width, 1152);
    assert_eq!(caixa(&dwindle, 2).width, 768);
}

#[test]
fn aumentar_a_janela_da_direita_empurra_a_divisao_para_a_esquerda() {
    let mut dwindle = arvore(&[1, 2]);
    dwindle.resize(janela(2), 192, 0, TELA);
    assert_eq!(caixa(&dwindle, 2), Rect::new(768, 0, 1152, 1080));
}

#[test]
fn a_altura_usa_a_divisao_empilhada_mais_proxima() {
    let mut dwindle = arvore(&[1, 2, 3]);
    dwindle.resize(janela(3), 0, 108, TELA);
    assert_eq!(caixa(&dwindle, 3).height, 648);
    assert_eq!(caixa(&dwindle, 1).height, 1080);
}

#[test]
fn a_proporcao_tem_limite() {
    let mut dwindle = arvore(&[1, 2]);
    dwindle.resize(janela(1), 5000, 0, TELA);
    assert_eq!(caixa(&dwindle, 1).width, 1728);
}

#[test]
fn sem_divisao_no_eixo_o_tamanho_nao_muda() {
    let mut dwindle = arvore(&[1, 2]);
    dwindle.resize(janela(1), 0, 200, TELA);
    assert_eq!(caixa(&dwindle, 1), Rect::new(0, 0, 960, 1080));
}

#[test]
fn trocar_a_folha_poe_a_janela_nova_no_mesmo_quadro() {
    let area = Rect::new(0, 0, 1000, 500);
    let mut dwindle = Dwindle::default();
    dwindle.insert(WindowId(1), None, area);
    dwindle.insert(WindowId(2), Some(WindowId(1)), area);
    let antes = dwindle.layout(area);
    assert!(dwindle.replace(WindowId(2), WindowId(9)));
    assert_eq!(dwindle.layout(area), vec![(WindowId(1), antes[0].1), (WindowId(9), antes[1].1)]);
    assert!(!dwindle.replace(WindowId(2), WindowId(5)));
    assert!(!dwindle.replace(WindowId(1), WindowId(9)));
}
