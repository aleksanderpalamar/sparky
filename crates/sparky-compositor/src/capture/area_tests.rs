use sparky_ipc::CaptureArea;
use sparky_tiling::Rect;

use crate::capture::area::{CaptureError, Target, resolve};

fn telas() -> Vec<(String, Rect)> {
    vec![
        ("HDMI-A-1".to_owned(), Rect::new(0, 0, 1920, 1080)),
        ("DP-1".to_owned(), Rect::new(1920, 0, 2560, 1440)),
    ]
}

fn alvo(output: &str, region: Rect) -> Result<Target, CaptureError> {
    Ok(Target {
        output: output.to_owned(),
        region,
    })
}

#[test]
fn o_monitor_sem_nome_e_o_em_foco_inteiro() {
    let area = CaptureArea::Output { output: None };
    assert_eq!(resolve(&area, &telas(), Some("DP-1"), None), alvo("DP-1", Rect::new(0, 0, 2560, 1440)));
}

#[test]
fn o_monitor_pelo_nome() {
    let area = CaptureArea::Output { output: Some("HDMI-A-1".to_owned()) };
    assert_eq!(resolve(&area, &telas(), Some("DP-1"), None), alvo("HDMI-A-1", Rect::new(0, 0, 1920, 1080)));
}

#[test]
fn monitor_desconhecido_e_erro() {
    let area = CaptureArea::Output { output: Some("VGA-9".to_owned()) };
    assert_eq!(resolve(&area, &telas(), None, None), Err(CaptureError::NoOutput));
}

#[test]
fn a_janela_vira_regiao_do_monitor_onde_ela_esta() {
    let janela = Rect::new(2000, 100, 800, 600);
    assert_eq!(
        resolve(&CaptureArea::Window, &telas(), Some("HDMI-A-1"), Some(janela)),
        alvo("DP-1", Rect::new(80, 100, 800, 600))
    );
}

#[test]
fn sem_janela_em_foco_e_erro() {
    assert_eq!(resolve(&CaptureArea::Window, &telas(), Some("DP-1"), None), Err(CaptureError::NoWindow));
}

#[test]
fn a_regiao_e_relativa_ao_monitor_e_cortada_na_borda() {
    let area = CaptureArea::Region { output: None, x: 1800, y: 1000, width: 400, height: 400 };
    assert_eq!(resolve(&area, &telas(), Some("HDMI-A-1"), None), alvo("HDMI-A-1", Rect::new(1800, 1000, 120, 80)));
}

#[test]
fn regiao_fora_do_monitor_ou_vazia_e_erro() {
    let fora = CaptureArea::Region { output: None, x: 5000, y: 0, width: 10, height: 10 };
    let vazia = CaptureArea::Region { output: None, x: 0, y: 0, width: 0, height: 10 };
    assert_eq!(resolve(&fora, &telas(), Some("HDMI-A-1"), None), Err(CaptureError::EmptyRegion));
    assert_eq!(resolve(&vazia, &telas(), Some("HDMI-A-1"), None), Err(CaptureError::EmptyRegion));
}
