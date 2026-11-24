# layershellev 0.19.1 (cópia com correções)

Cópia do crate [`layershellev` 0.19.1](https://crates.io/crates/layershellev/0.19.1)
(licença MIT, autores do projeto waycrate/exwlshelleventloop), usada via
`[patch.crates-io]` no `Cargo.toml` da raiz. É a base da libsparky para menus
como popups reais.

Mudanças:

- `NewXdgWindowSettings` ganhou `app_id` e `min_size`, enviados ao compositor
  na criação da janela (`set_app_id`, `set_min_size`).
- Popups aceitam uma janela xdg (`Shell::XdgTopLevel`) como pai, além de
  camadas e outros popups.

Remover quando uma versão publicada trouxer essas capacidades.
