# Sparky

Ambiente de desktop Wayland escrito em Rust, com compositor próprio, feito para ser usado pelo teclado. O mouse funciona, mas tudo tem atalho.

O layout vem do [Omarchy](https://github.com/basecamp/omarchy): barra no topo com as áreas de trabalho à esquerda, o relógio no centro e a bandeja à direita; janelas lado a lado com as mesmas folgas, bordas e atalhos.

## Organização

| Caminho | O que é |
|---|---|
| `crates/sparky-compositor` | O compositor Wayland (binário `sparky`), feito com a biblioteca Smithay: protocolos, janelas, bordas, teclado e ponteiro |
| `crates/sparky-tiling` | O mosaico: árvore dwindle, folgas, foco por direção, flutuantes, tela cheia, áreas de trabalho e monitores. Não depende de Wayland |
| `crates/sparky-theme` | Paleta de cores compartilhada pelos aplicativos, lida do `colors.toml` do tema |
| `data/themes` | Temas no formato `colors.toml` |

## Mosaico

Cada janela nova divide a caixa da janela em foco: lado a lado se a caixa for mais larga que alta, empilhada se não, e a nova fica à direita ou embaixo. A divisão escolhida é preservada quando a caixa muda de tamanho e pode ser invertida à mão. Fechar uma janela devolve o espaço para a vizinha. Entre janelas a folga é de 5 px de cada lado, e de 10 px até a borda da tela (`Gaps::SPARKY`).

As áreas de trabalho vão de 1 a 10 e cada uma pertence a um monitor. Ir para uma área visível em outro monitor leva o foco até ele; uma área vazia que sai da tela deixa de existir.

## Compositor

O `sparky` é o próprio compositor Wayland; não depende de nenhum outro. Fala xdg-shell (janelas), wlr-layer-shell (barra, lançador, notificações), decorações do lado do servidor, área de transferência (incluindo seleção primária e wlr-data-control) e viewporter.

Toda janela comum entra no mosaico. Flutuam as que têm janela-mãe (diálogos), as de tamanho fixo e as do painel de configurações e do capturador. A borda da janela em foco usa a cor de destaque do tema, e o foco do teclado acompanha o ponteiro.

Rodado num terminal virtual (ou pelo gerenciador de login), o `sparky` assume a placa de vídeo pela sessão do sistema (libseat), liga cada monitor conectado lado a lado da esquerda para a direita, acompanha monitor ligado e desligado em tempo real, lê teclado, mouse e touchpad pelo libinput (toque para clicar ligado) e desenha o cursor do tema `XCURSOR_THEME`. `Ctrl+Alt+F1…F12` troca de terminal virtual.

Para testar dentro de outra sessão Wayland, o `sparky` abre numa janela:

```bash
cargo run -p sparky-compositor -- --spawn kitty
```

`--spawn` pode se repetir e abre cada comando já dentro do sparky. O layout do teclado vem do `/etc/vconsole.conf` ou, no Debian, do `/etc/default/keyboard`.

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
