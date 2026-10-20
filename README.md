# Sparky

Ambiente de desktop sobre o Hyprland, feito para ser usado pelo teclado. O mouse funciona, mas tudo tem atalho.

O layout vem do [Omarchy](https://github.com/basecamp/omarchy): barra no topo com as áreas de trabalho à esquerda, o relógio no centro e a bandeja à direita; janelas lado a lado com as mesmas folgas, bordas e atalhos.

## Organização

| Caminho | O que é |
|---|---|
| `crates/sparky-theme` | Paleta de cores compartilhada pelos aplicativos, lida do `colors.toml` do tema |
| `data/themes` | Temas no formato `colors.toml` do Omarchy |

## Tema

As cores saem do primeiro arquivo que existir:

1. `~/.config/sparky/colors.toml`
2. `$SPARKY_PATH/themes/tokyo-night/colors.toml` (`SPARKY_PATH` é `/usr/share/sparky` quando instalado)

Sem nenhum dos dois, vale o Tokyo Night embutido.

## Desenvolvimento

```bash
cargo test
cargo clippy --all-targets
```

## Créditos

O tema Tokyo Night (`data/themes/tokyo-night/colors.toml`) e o layout vêm do Omarchy, © David Heinemeier Hansson, licença MIT.
