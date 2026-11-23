use crate::glass::glass_in;

#[test]
fn le_frosted_glass_da_secao_general_do_sparky_toml() {
    assert!(glass_in("[general]\nfrosted_glass = true\n"));
    assert!(!glass_in("[general]\nfrosted_glass = false\n"));
    assert!(!glass_in(""));
    assert!(!glass_in("[dock]\nfrosted_glass = true\n"));
    assert!(!glass_in("isto não é toml ["));
}
