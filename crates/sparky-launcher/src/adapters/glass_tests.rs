use sparky_ipc::GlassArea;

use super::glass::area;
use crate::domain::mode::Mode;

#[test]
fn o_vidro_cobre_so_o_painel_centralizado_de_cada_modo() {
    assert_eq!(
        area(Mode::Apps.size()),
        GlassArea::Centered {
            width: 800,
            height: 600
        }
    );
    assert_eq!(
        area(Mode::System.size()),
        GlassArea::Centered {
            width: 440,
            height: 252
        }
    );
}
