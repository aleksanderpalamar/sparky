#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate<C, K> {
    pub connector: C,
    pub crtcs: Vec<K>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan<C, K> {
    pub connect: Vec<(C, K)>,
    pub disconnect: Vec<C>,
}

pub fn plan<C: Copy + Eq, K: Copy + Eq>(connected: &[Candidate<C, K>], assigned: &[(C, K)]) -> Plan<C, K> {
    let disconnect: Vec<C> = assigned
        .iter()
        .filter(|(connector, _)| !connected.iter().any(|candidate| candidate.connector == *connector))
        .map(|(connector, _)| *connector)
        .collect();
    let mut taken: Vec<K> = assigned
        .iter()
        .filter(|(connector, _)| !disconnect.contains(connector))
        .map(|(_, crtc)| *crtc)
        .collect();
    let mut connect = Vec::new();
    for candidate in connected {
        if assigned.iter().any(|(connector, _)| *connector == candidate.connector) {
            continue;
        }
        if let Some(crtc) = candidate.crtcs.iter().find(|crtc| !taken.contains(crtc)).copied() {
            taken.push(crtc);
            connect.push((candidate.connector, crtc));
        }
    }
    Plan { connect, disconnect }
}

#[cfg(test)]
mod tests {
    use super::{Candidate, Plan, plan};

    fn conector(connector: u32, crtcs: &[u32]) -> Candidate<u32, u32> {
        Candidate {
            connector,
            crtcs: crtcs.to_vec(),
        }
    }

    #[test]
    fn conector_novo_pega_o_primeiro_crtc_livre() {
        assert_eq!(
            plan(&[conector(1, &[10, 11])], &[]),
            Plan {
                connect: vec![(1, 10)],
                disconnect: vec![]
            }
        );
    }

    #[test]
    fn dois_conectores_nao_dividem_o_mesmo_crtc() {
        let found = plan(&[conector(1, &[10, 11]), conector(2, &[10, 11])], &[]);
        assert_eq!(found.connect, vec![(1, 10), (2, 11)]);
    }

    #[test]
    fn conector_ja_ligado_fica_como_esta() {
        let found = plan(&[conector(1, &[10, 11]), conector(2, &[10, 11])], &[(1, 11)]);
        assert_eq!(found.connect, vec![(2, 10)]);
        assert!(found.disconnect.is_empty());
    }

    #[test]
    fn conector_que_sumiu_e_desligado_e_libera_o_crtc() {
        let found = plan(&[conector(2, &[10])], &[(1, 10)]);
        assert_eq!(found.disconnect, vec![1]);
        assert_eq!(found.connect, vec![(2, 10)]);
    }

    #[test]
    fn sem_crtc_livre_o_conector_espera() {
        let found = plan(&[conector(1, &[10]), conector(2, &[10])], &[]);
        assert_eq!(found.connect, vec![(1, 10)]);
    }
}
