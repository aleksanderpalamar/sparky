const HEADER: [u8; 8] = [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00];
const DESCRIPTORS: [usize; 4] = [54, 72, 90, 108];
const DESCRIPTOR_SIZE: usize = 18;
const MONITOR_NAME: u8 = 0xfc;

pub fn describe(edid: &[u8]) -> Option<String> {
    if edid.len() < 128 || edid[..8] != HEADER {
        return None;
    }
    let make = manufacturer(u16::from_be_bytes([edid[8], edid[9]]));
    let name = DESCRIPTORS
        .iter()
        .find_map(|offset| monitor_name(&edid[*offset..*offset + DESCRIPTOR_SIZE]));
    Some(match name {
        Some(name) => format!("{make} {name}"),
        None => make,
    })
}

fn manufacturer(code: u16) -> String {
    [10, 5, 0]
        .into_iter()
        .map(|shift| match ((code >> shift) & 0x1f) as u8 {
            letter @ 1..=26 => char::from(b'A' + letter - 1),
            _ => '?',
        })
        .collect()
}

fn monitor_name(block: &[u8]) -> Option<String> {
    if block[..3] != [0, 0, 0] || block[3] != MONITOR_NAME {
        return None;
    }
    let text = String::from_utf8_lossy(&block[5..]);
    let name = text.split('\n').next().unwrap_or_default().trim();
    (!name.is_empty()).then(|| name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::describe;

    fn edid(make: u16, name: Option<&str>) -> Vec<u8> {
        let mut bytes = vec![0u8; 128];
        bytes[..8].copy_from_slice(&super::HEADER);
        bytes[8..10].copy_from_slice(&make.to_be_bytes());
        if let Some(name) = name {
            let block = &mut bytes[72..90];
            block[3] = 0xfc;
            let mut text = name.as_bytes().to_vec();
            text.push(b'\n');
            text.resize(13, b' ');
            block[5..].copy_from_slice(&text);
        }
        bytes
    }

    #[test]
    fn junta_fabricante_e_nome_do_monitor() {
        let dell = (4 << 10) | (5 << 5) | 12;
        assert_eq!(
            describe(&edid(dell, Some("U2723QE"))),
            Some("DEL U2723QE".to_owned())
        );
    }

    #[test]
    fn sem_nome_fica_so_o_fabricante() {
        let samsung = (19 << 10) | (1 << 5) | 13;
        assert_eq!(describe(&edid(samsung, None)), Some("SAM".to_owned()));
    }

    #[test]
    fn dados_que_nao_sao_edid_sao_ignorados() {
        assert_eq!(describe(&[0; 64]), None);
        assert_eq!(describe(&[1; 128]), None);
    }
}
