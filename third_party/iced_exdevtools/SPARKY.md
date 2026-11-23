# iced_exdevtools 0.19.1 (cópia com correção)

Cópia do crate [`iced_exdevtools` 0.19.1](https://crates.io/crates/iced_exdevtools/0.19.1)
(licença MIT, autores do projeto waycrate/exwlshelleventloop), usada via
`[patch.crates-io]` no `Cargo.toml` da raiz.

Única mudança: em `src/keymap.rs`, a conversão de `NativeKeyCode` ganhou o caso
`_ => Unidentified`, porque o `winit-core` 0.31.0-beta.3 (exigido pelo smithay
usado no compositor) tornou o enum não exaustivo.

Remover esta pasta e o `[patch.crates-io]` quando uma versão publicada do
`iced_layershell` aceitar o `winit-core` 0.31.0-beta.3.
