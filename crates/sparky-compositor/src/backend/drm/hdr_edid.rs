use crate::color_management::description::Range;

const BLOCK: usize = 128;
const CTA: u8 = 0x02;
const EXTENDED: u8 = 7;
const COLORIMETRY: u8 = 5;
const STATIC_METADATA: u8 = 6;
const BT2020_RGB: u8 = 0x80;
const SMPTE_ST2084: u8 = 0x04;
const FALLBACK_MAX_NITS: f64 = 1000.0;
const FALLBACK_MIN_NITS: f64 = 0.005;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HdrCapability {
    pub pq: bool,
    pub bt2020: bool,
    pub max_nits: Option<f64>,
    pub average_nits: Option<f64>,
    pub min_nits: Option<f64>,
}

impl HdrCapability {
    pub fn supported(&self) -> bool {
        self.pq && self.bt2020
    }

    pub fn peak_nits(&self) -> f64 {
        self.max_nits.unwrap_or(FALLBACK_MAX_NITS)
    }

    pub fn black_nits(&self) -> f64 {
        self.min_nits.unwrap_or(FALLBACK_MIN_NITS)
    }

    pub fn luminance(&self) -> Range {
        Range::from_nits(self.black_nits(), self.peak_nits())
    }
}

pub fn hdr_capability(edid: &[u8]) -> HdrCapability {
    let mut capability = HdrCapability::default();
    for block in edid
        .chunks_exact(BLOCK)
        .skip(1)
        .filter(|block| block[0] == CTA)
    {
        for (tag, payload) in data_blocks(block) {
            if tag == EXTENDED {
                read(&mut capability, payload);
            }
        }
    }
    capability
}

fn data_blocks(block: &[u8]) -> Vec<(u8, &[u8])> {
    let end = usize::from(block[2]).min(BLOCK - 1);
    let mut offset = 4;
    let mut blocks = Vec::new();
    while offset < end {
        let header = block[offset];
        let length = usize::from(header & 0x1f);
        let Some(payload) = block.get(offset + 1..offset + 1 + length) else {
            break;
        };
        blocks.push((header >> 5, payload));
        offset += 1 + length;
    }
    blocks
}

fn read(capability: &mut HdrCapability, payload: &[u8]) {
    let flags = |index: usize| payload.get(index).copied().unwrap_or_default();
    match payload.first().copied() {
        Some(COLORIMETRY) => capability.bt2020 = flags(1) & BT2020_RGB != 0,
        Some(STATIC_METADATA) => {
            capability.pq = flags(1) & SMPTE_ST2084 != 0;
            capability.max_nits = luminance(payload.get(3));
            capability.average_nits = luminance(payload.get(4));
            capability.min_nits = capability
                .max_nits
                .zip(payload.get(5))
                .map(|(max, code)| max * (f64::from(*code) / 255.0).powi(2) / 100.0);
        }
        _ => {}
    }
}

fn luminance(code: Option<&u8>) -> Option<f64> {
    code.filter(|code| **code > 0)
        .map(|code| 50.0 * 2f64.powf(f64::from(*code) / 32.0))
}

#[cfg(test)]
mod tests {
    use super::hdr_capability;
    use crate::color_management::description::Range;

    fn edid(blocks: &[&[u8]]) -> Vec<u8> {
        let mut bytes = vec![0u8; 256];
        bytes[126] = 1;
        let cta = &mut bytes[128..];
        cta[0] = 0x02;
        cta[1] = 0x03;
        let mut offset = 4;
        for block in blocks {
            cta[offset..offset + block.len()].copy_from_slice(block);
            offset += block.len();
        }
        cta[2] = offset as u8;
        bytes
    }

    const COLORIMETRIA: [u8; 4] = [0xe3, 0x05, 0xc0, 0x00];

    #[test]
    fn tv_com_pq_e_bt2020_suporta_hdr10() {
        let hdr = [0xe6, 0x06, 0x05, 0x01, 0x78, 0x5a, 0x10];
        let capability = hdr_capability(&edid(&[&COLORIMETRIA, &hdr]));
        assert!(capability.supported());
        let max = capability.max_nits.unwrap_or_default();
        assert!((max - 50.0 * 2f64.powf(120.0 / 32.0)).abs() < 0.01);
        assert!(capability.average_nits.is_some_and(|average| average < max));
        assert!(
            capability
                .min_nits
                .is_some_and(|min| min > 0.0 && min < 1.0)
        );
    }

    #[test]
    fn sem_luminancias_o_monitor_continua_suportado() {
        let hdr = [0xe3, 0x06, 0x05, 0x01];
        let capability = hdr_capability(&edid(&[&hdr, &COLORIMETRIA]));
        assert!(capability.supported());
        assert_eq!(capability.max_nits, None);
        assert_eq!(capability.min_nits, None);
    }

    #[test]
    fn brilho_da_tela_vem_do_edid_ou_do_padrao_hdr10() {
        let hdr = [0xe6, 0x06, 0x05, 0x01, 0x78, 0x5a, 0x10];
        let tv = hdr_capability(&edid(&[&COLORIMETRIA, &hdr]));
        let max = tv.max_nits.unwrap_or_default();
        let min = tv.min_nits.unwrap_or_default();
        assert_eq!(tv.luminance(), Range::from_nits(min, max));
        let sem_dados = hdr_capability(&edid(&[&[0xe3, 0x06, 0x05, 0x01], &COLORIMETRIA]));
        assert_eq!(
            sem_dados.luminance(),
            Range {
                min: 50,
                max: 1_000
            }
        );
    }

    #[test]
    fn faltar_pq_ou_bt2020_nao_e_hdr10() {
        let hlg = [0xe3, 0x06, 0x09, 0x01];
        assert!(!hdr_capability(&edid(&[&hlg, &COLORIMETRIA])).supported());
        let pq = [0xe3, 0x06, 0x05, 0x01];
        assert!(!hdr_capability(&edid(&[&pq])).supported());
        assert!(!hdr_capability(&[0u8; 128]).supported());
    }
}
