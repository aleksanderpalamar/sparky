use smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_manager_v1::{
    Primaries, TransferFunction as Transfer,
};
use smithay::reexports::wayland_server::WEnum;

use crate::color_management::description::{NamedPrimaries, TransferFunction};

const COMPOUND_POWER_SINCE: u32 = 2;

pub const fn transfer(value: TransferFunction, version: u32) -> Transfer {
    match value {
        TransferFunction::Srgb if version >= COMPOUND_POWER_SINCE => Transfer::CompoundPower24,
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

pub fn supported_transfer(value: WEnum<Transfer>, version: u32) -> Option<TransferFunction> {
    TransferFunction::ALL
        .into_iter()
        .find(|known| value == WEnum::Value(transfer(*known, version)))
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

    use super::{identity_parts, supported_primaries, supported_transfer, transfer};
    use crate::color_management::description::{NamedPrimaries, TransferFunction};

    #[test]
    fn so_curvas_e_primarias_anunciadas_sao_aceitas() {
        assert_eq!(
            supported_transfer(WEnum::Value(Transfer::St2084Pq), 2),
            Some(TransferFunction::St2084Pq)
        );
        assert_eq!(supported_transfer(WEnum::Value(Transfer::Bt1886), 2), None);
        assert_eq!(supported_transfer(WEnum::Unknown(99), 2), None);
        assert_eq!(
            supported_primaries(WEnum::Value(Primaries::DisplayP3)),
            Some(NamedPrimaries::DisplayP3)
        );
        assert_eq!(supported_primaries(WEnum::Value(Primaries::AdobeRgb)), None);
    }

    #[test]
    fn curva_srgb_tem_o_nome_da_versao_do_cliente() {
        assert_eq!(transfer(TransferFunction::Srgb, 1), Transfer::Srgb);
        assert_eq!(
            transfer(TransferFunction::Srgb, 2),
            Transfer::CompoundPower24
        );
        assert_eq!(transfer(TransferFunction::Gamma22, 2), Transfer::Gamma22);
        let srgb = TransferFunction::Srgb;
        let antiga = WEnum::Value(Transfer::Srgb);
        let nova = WEnum::Value(Transfer::CompoundPower24);
        assert_eq!(supported_transfer(antiga, 1), Some(srgb));
        assert_eq!(supported_transfer(nova, 1), None);
        assert_eq!(supported_transfer(antiga, 2), None);
        assert_eq!(supported_transfer(nova, 2), Some(srgb));
    }

    #[test]
    fn identidade_vira_metade_alta_e_baixa() {
        assert_eq!(identity_parts(7), (0, 7));
        assert_eq!(identity_parts((3 << 32) | 9), (3, 9));
    }
}
