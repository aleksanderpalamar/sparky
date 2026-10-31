use crate::backend::drm::hdr_edid::HdrCapability;

pub const SIZE: usize = 32;
const STATIC_TYPE_1: u32 = 0;
const EOTF_SMPTE_ST2084: u8 = 2;
const CHROMATICITY_UNIT: f64 = 0.00002;
const MIN_LUMINANCE_UNIT: f64 = 0.0001;
const PRIMARIES: [(f64, f64); 4] = [
    (0.708, 0.292),
    (0.170, 0.797),
    (0.131, 0.046),
    (0.3127, 0.3290),
];
const FALLBACK_MAX_NITS: f64 = 1000.0;
const FALLBACK_MIN_NITS: f64 = 0.005;

pub fn hdr10(capability: &HdrCapability) -> [u8; SIZE] {
    let mut bytes = [0u8; SIZE];
    bytes[..4].copy_from_slice(&STATIC_TYPE_1.to_ne_bytes());
    bytes[4] = EOTF_SMPTE_ST2084;
    for (index, (x, y)) in PRIMARIES.into_iter().enumerate() {
        put(&mut bytes, 6 + index * 4, units(x, CHROMATICITY_UNIT));
        put(&mut bytes, 8 + index * 4, units(y, CHROMATICITY_UNIT));
    }
    let max = capability.max_nits.unwrap_or(FALLBACK_MAX_NITS);
    let min = capability.min_nits.unwrap_or(FALLBACK_MIN_NITS);
    put(&mut bytes, 22, units(max, 1.0));
    put(&mut bytes, 24, units(min, MIN_LUMINANCE_UNIT));
    put(&mut bytes, 26, units(max, 1.0));
    put(
        &mut bytes,
        28,
        units(capability.average_nits.unwrap_or(max), 1.0),
    );
    bytes
}

fn units(value: f64, unit: f64) -> u16 {
    (value / unit).round().clamp(0.0, f64::from(u16::MAX)) as u16
}

fn put(bytes: &mut [u8; SIZE], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_ne_bytes());
}

#[cfg(test)]
mod tests {
    use super::{SIZE, hdr10};
    use crate::backend::drm::hdr_edid::HdrCapability;

    fn campo(bytes: &[u8; SIZE], offset: usize) -> u16 {
        u16::from_ne_bytes([bytes[offset], bytes[offset + 1]])
    }

    #[test]
    fn pacote_tem_o_layout_do_kernel_com_pq_e_bt2020() {
        let bytes = hdr10(&HdrCapability::default());
        assert_eq!(
            u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            0
        );
        assert_eq!(bytes[4], 2);
        assert_eq!(bytes[5], 0);
        assert_eq!((campo(&bytes, 6), campo(&bytes, 8)), (35400, 14600));
        assert_eq!((campo(&bytes, 10), campo(&bytes, 12)), (8500, 39850));
        assert_eq!((campo(&bytes, 14), campo(&bytes, 16)), (6550, 2300));
        assert_eq!((campo(&bytes, 18), campo(&bytes, 20)), (15635, 16450));
        assert_eq!(&bytes[30..], &[0, 0]);
    }

    #[test]
    fn luminancias_vem_do_edid_ou_do_padrao() {
        let padrao = hdr10(&HdrCapability::default());
        assert_eq!((campo(&padrao, 22), campo(&padrao, 24)), (1000, 50));
        assert_eq!((campo(&padrao, 26), campo(&padrao, 28)), (1000, 1000));
        let tv = HdrCapability {
            max_nits: Some(672.7),
            average_nits: Some(400.2),
            min_nits: Some(0.05),
            ..HdrCapability::default()
        };
        let bytes = hdr10(&tv);
        assert_eq!((campo(&bytes, 22), campo(&bytes, 24)), (673, 500));
        assert_eq!((campo(&bytes, 26), campo(&bytes, 28)), (673, 400));
    }
}
