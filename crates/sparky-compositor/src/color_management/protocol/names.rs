use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_manager_v1::{
    Primaries, TransferFunction as Transfer,
};
use smithay::reexports::wayland_server::WEnum;

use crate::color_management::description::{NamedPrimaries, TransferFunction};

pub const fn transfer(value: TransferFunction) -> Transfer {
    match value {
        TransferFunction::Srgb => Transfer::Srgb,
        TransferFunction::Gamma22 => Transfer::Gamma22,
        TransferFunction::ExtLinear => Transfer::ExtLinear,
        TransferFunction::St2084Pq => Transfer::St2084Pq,
    }
}

pub const fn primaries(value: NamedPrimaries) -> Primaries {
    match value {
        NamedPrimaries::Srgb => Primaries::Srgb,
        NamedPrimaries::Bt2020 => Primaries::Bt2020,
        NamedPrimaries::DisplayP3 => Primaries::DisplayP3,
        NamedPrimaries::DciP3 => Primaries::DciP3,
    }
}

pub fn supported_transfer(value: WEnum<Transfer>) -> Option<TransferFunction> {
    TransferFunction::ALL
        .into_iter()
        .find(|known| value == WEnum::Value(transfer(*known)))
}

pub fn supported_primaries(value: WEnum<Primaries>) -> Option<NamedPrimaries> {
    NamedPrimaries::ALL
        .into_iter()
        .find(|known| value == WEnum::Value(primaries(*known)))
}

pub const fn identity_parts(identity: u64) -> (u32, u32) {
    ((identity >> 32) as u32, identity as u32)
}

#[cfg(test)]
mod tests {
    use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_manager_v1::{
        Primaries, TransferFunction as Transfer,
    };
    use smithay::reexports::wayland_server::WEnum;

    use super::{identity_parts, supported_primaries, supported_transfer};
    use crate::color_management::description::{NamedPrimaries, TransferFunction};

    #[test]
    fn so_curvas_e_primarias_anunciadas_sao_aceitas() {
        assert_eq!(
            supported_transfer(WEnum::Value(Transfer::St2084Pq)),
            Some(TransferFunction::St2084Pq)
        );
        assert_eq!(supported_transfer(WEnum::Value(Transfer::Bt1886)), None);
        assert_eq!(supported_transfer(WEnum::Unknown(99)), None);
        assert_eq!(
            supported_primaries(WEnum::Value(Primaries::DisplayP3)),
            Some(NamedPrimaries::DisplayP3)
        );
        assert_eq!(supported_primaries(WEnum::Value(Primaries::AdobeRgb)), None);
    }

    #[test]
    fn identidade_vira_metade_alta_e_baixa() {
        assert_eq!(identity_parts(7), (0, 7));
        assert_eq!(identity_parts((3 << 32) | 9), (3, 9));
    }
}
