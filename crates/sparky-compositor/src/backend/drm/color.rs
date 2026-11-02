use smithay::output::Output;

use crate::backend::drm::Drm;
use crate::backend::drm::hdr_state::HdrState;
use crate::color_management::presets::OutputColor;

impl Drm {
    pub fn output_colors(&self) -> Vec<(Output, OutputColor)> {
        self.gpu
            .surfaces
            .values()
            .map(|surface| (surface.output.clone(), color(surface.hdr)))
            .collect()
    }
}

fn color(hdr: HdrState) -> OutputColor {
    match hdr {
        HdrState::On { display, .. } => OutputColor::Hdr(display),
        HdrState::Off | HdrState::Pending => OutputColor::Sdr,
    }
}

#[cfg(test)]
mod tests {
    use super::color;
    use crate::backend::drm::hdr_props::exemplo;
    use crate::backend::drm::hdr_state::HdrState;
    use crate::color_management::description::Range;
    use crate::color_management::presets::OutputColor;

    #[test]
    fn so_o_hdr_ligado_descreve_a_tela_em_hdr() {
        let tela = Range { min: 50, max: 600 };
        let ligado = HdrState::On {
            properties: exemplo(),
            blob: 3,
            display: tela,
        };
        assert_eq!(color(ligado), OutputColor::Hdr(tela));
        assert_eq!(color(HdrState::Pending), OutputColor::Sdr);
        assert_eq!(color(HdrState::Off), OutputColor::Sdr);
    }
}
