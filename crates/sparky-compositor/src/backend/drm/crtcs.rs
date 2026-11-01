#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate<C, K> {
    pub connector: C,
    pub crtcs: Vec<K>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan<C, K> {
    pub connect: Vec<(C, K)>,
    pub disconnect: Vec<C>,
    pub release: Vec<C>,
}

pub fn plan<C: Copy + Eq, K: Copy + Eq>(
    connected: &[Candidate<C, K>],
    present: &[C],
    assigned: &[(C, K)],
) -> Plan<C, K> {
    let unwanted = |connector: &C| {
        !connected
            .iter()
            .any(|candidate| candidate.connector == *connector)
    };
    let (release, disconnect): (Vec<C>, Vec<C>) = assigned
        .iter()
        .map(|(connector, _)| *connector)
        .filter(unwanted)
        .partition(|connector| present.contains(connector));
    let mut taken: Vec<K> = assigned
        .iter()
        .filter(|(connector, _)| !disconnect.contains(connector))
        .map(|(_, crtc)| *crtc)
        .collect();
    let mut connect = Vec::new();
    for candidate in connected {
        if assigned
            .iter()
            .any(|(connector, _)| *connector == candidate.connector)
        {
            continue;
        }
        if let Some(crtc) = free(&candidate.crtcs, &taken) {
            taken.push(crtc);
            connect.push((candidate.connector, crtc));
        }
    }
    Plan {
        connect,
        disconnect,
        release,
    }
}

pub fn releasable<C: Copy>(release: &[C], active: usize) -> Vec<C> {
    release
        .iter()
        .copied()
        .take(active.saturating_sub(1))
        .collect()
}

fn free<K: Copy + Eq>(crtcs: &[K], taken: &[K]) -> Option<K> {
    crtcs.iter().find(|crtc| !taken.contains(crtc)).copied()
}

#[cfg(test)]
mod tests {
    use super::{Candidate, Plan, plan, releasable};

    fn conector(connector: u32, crtcs: &[u32]) -> Candidate<u32, u32> {
        Candidate {
            connector,
            crtcs: crtcs.to_vec(),
        }
    }

    #[test]
    fn conector_novo_pega_o_primeiro_crtc_livre() {
        assert_eq!(
            plan(&[conector(1, &[10, 11])], &[1], &[]),
            Plan {
                connect: vec![(1, 10)],
                disconnect: vec![],
                release: vec![],
            }
        );
    }

    #[test]
    fn dois_conectores_nao_dividem_o_mesmo_crtc() {
        let found = plan(
            &[conector(1, &[10, 11]), conector(2, &[10, 11])],
            &[1, 2],
            &[],
        );
        assert_eq!(found.connect, vec![(1, 10), (2, 11)]);
    }

    #[test]
    fn conector_ja_ligado_fica_como_esta() {
        let found = plan(
            &[conector(1, &[10, 11]), conector(2, &[10, 11])],
            &[1, 2],
            &[(1, 11)],
        );
        assert_eq!(found.connect, vec![(2, 10)]);
        assert!(found.disconnect.is_empty());
    }

    #[test]
    fn conector_que_sumiu_e_desligado_e_libera_o_crtc() {
        let found = plan(&[conector(2, &[10])], &[2], &[(1, 10)]);
        assert_eq!(found.disconnect, vec![1]);
        assert_eq!(found.connect, vec![(2, 10)]);
    }

    #[test]
    fn sem_crtc_livre_o_conector_espera() {
        let found = plan(&[conector(1, &[10]), conector(2, &[10])], &[1, 2], &[]);
        assert_eq!(found.connect, vec![(1, 10)]);
    }

    #[test]
    fn tela_desligada_na_configuracao_espera_outra_ligar() {
        let found = plan(&[conector(2, &[])], &[1, 2], &[(1, 10)]);
        assert_eq!(
            found,
            Plan {
                connect: vec![],
                disconnect: vec![],
                release: vec![1],
            }
        );
        assert!(releasable(&found.release, 1).is_empty());
    }

    #[test]
    fn tela_desligada_guarda_o_crtc_ate_ser_solta() {
        let found = plan(&[conector(2, &[10, 11])], &[1, 2], &[(1, 10)]);
        assert_eq!(found.connect, vec![(2, 11)]);
        assert_eq!(found.release, vec![1]);
        assert_eq!(releasable(&found.release, 2), vec![1]);
    }

    #[test]
    fn soltar_telas_sempre_deixa_uma_ligada() {
        assert_eq!(releasable(&[1, 2], 3), vec![1, 2]);
        assert_eq!(releasable(&[1, 2], 2), vec![1]);
        assert!(releasable(&[1], 0).is_empty());
    }
}
