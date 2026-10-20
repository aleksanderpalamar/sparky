# Sparky

Ambiente de desktop Wayland escrito em Rust, com compositor próprio, feito para ser usado pelo teclado. O mouse funciona, mas tudo tem atalho.

O layout vem do [Omarchy](https://github.com/basecamp/omarchy): barra no topo com as áreas de trabalho à esquerda, o relógio no centro e a bandeja à direita; janelas lado a lado com as mesmas folgas, bordas e atalhos.

## Organização

| Caminho | O que é |
|---|---|
| `crates/sparky-tiling` | O mosaico: árvore dwindle, folgas, foco por direção, flutuantes, tela cheia, áreas de trabalho e monitores. Não depende de Wayland |
| `crates/sparky-theme` | Paleta de cores compartilhada pelos aplicativos, lida do `colors.toml` do tema |
| `data/themes` | Temas no formato `colors.toml` |

## Mosaico

Cada janela nova divide a caixa da janela em foco: lado a lado se a caixa for mais larga que alta, empilhada se não, e a nova fica à direita ou embaixo. A divisão escolhida é preservada quando a caixa muda de tamanho e pode ser invertida à mão. Fechar uma janela devolve o espaço para a vizinha. Entre janelas a folga é de 5 px de cada lado, e de 10 px até a borda da tela (`Gaps::SPARKY`).

As áreas de trabalho vão de 1 a 10 e cada uma pertence a um monitor. Ir para uma área visível em outro monitor leva o foco até ele; uma área vazia que sai da tela deixa de existir.

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
