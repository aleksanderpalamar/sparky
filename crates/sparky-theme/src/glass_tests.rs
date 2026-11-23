use crate::glass::glass_in;

#[test]
fn le_frosted_glass_da_secao_general_do_sparky_toml() {
    assert!(glass_in("[general]\nfrosted_glass = true\n"));
    assert!(!glass_in("[general]\nfrosted_glass = false\n"));
    assert!(!glass_in(""));
    assert!(!glass_in("[dock]\nfrosted_glass = true\n"));
    assert!(!glass_in("isto não é toml ["));
}

#[test]
fn o_vidro_nao_muda_a_identidade_do_tema() {
    let vidro = crate::Palette {
        glass: true,
        ..crate::Palette::SPARKY
    };
    assert_eq!(vidro, crate::Palette::SPARKY);
    assert_ne!(
        crate::Palette {
            glass: true,
            ..crate::Palette::TOKYO_NIGHT
        },
        crate::Palette::SPARKY
    );
}
