# Sparky

Ambiente de desktop Wayland escrito em Rust, com compositor próprio, feito para ser usado pelo teclado. O mouse funciona, mas tudo tem atalho.

O layout vem do [Omarchy](https://github.com/basecamp/omarchy): barra no topo com as áreas de trabalho à esquerda, o relógio no centro e a bandeja à direita; janelas lado a lado com as mesmas folgas, bordas e atalhos.

## Organização

| Caminho | O que é |
|---|---|
| `crates/sparky-bar` | A barra do topo (`sparky-bar`): áreas de trabalho, relógio e bandeja, uma por monitor |
| `crates/sparky-compositor` | O compositor Wayland (binário `sparky`), feito com a biblioteca Smithay: protocolos, janelas, bordas, teclado e ponteiro |
| `crates/sparky-tiling` | O mosaico: árvore dwindle, folgas, foco por direção, flutuantes, tela cheia, áreas de trabalho e monitores. Não depende de Wayland |
| `crates/sparky-config` | O `sparky.toml`: esquema, padrões do sparky, ações e a tabela de atalhos padrão |
| `crates/sparky-ipc` | Protocolo do IPC do sparky (JSON por linha num socket Unix), o cliente usado pela barra e pelos aplicativos, e o `sparky-msg` |
| `crates/sparky-theme` | Paleta de cores compartilhada pelos aplicativos, lida do `colors.toml` do tema |
| `data/themes` | Temas no formato `colors.toml` |

## Mosaico

Cada janela nova divide a caixa da janela em foco: lado a lado se a caixa for mais larga que alta, empilhada se não, e a nova fica à direita ou embaixo. A divisão escolhida é preservada quando a caixa muda de tamanho e pode ser invertida à mão. Fechar uma janela devolve o espaço para a vizinha. Entre janelas a folga é de 5 px de cada lado, e de 10 px até a borda da tela (`Gaps::SPARKY`).

As áreas de trabalho vão de 1 a 10 e cada uma pertence a um monitor. Ir para uma área visível em outro monitor leva o foco até ele; uma área vazia que sai da tela deixa de existir.

## Compositor

O `sparky` é o próprio compositor Wayland; não depende de nenhum outro. Fala xdg-shell (janelas), wlr-layer-shell (barra, lançador, notificações), decorações do lado do servidor, área de transferência (incluindo seleção primária e wlr-data-control) e viewporter.

Aplicativos X11 rodam pelo XWayland, que o sparky sobe sozinho e controla com o gerenciador de janelas X11 do Smithay: janelas X11 entram no mosaico como as Wayland, menus e dicas X11 (janelas sem gerência) ficam por cima de tudo, e a área de transferência e a seleção primária passam nos dois sentidos. Os programas abertos pelo sparky recebem `DISPLAY` e `WAYLAND_DISPLAY`.

Toda janela comum entra no mosaico. Flutuam as que têm janela-mãe (diálogos), as de tamanho fixo e as do painel de configurações e do capturador. A borda da janela em foco usa a cor de destaque do tema, e o foco do teclado acompanha o ponteiro.

Rodado num terminal virtual (ou pelo gerenciador de login), o `sparky` assume a placa de vídeo pela sessão do sistema (libseat), liga cada monitor conectado lado a lado da esquerda para a direita, acompanha monitor ligado e desligado em tempo real, lê teclado, mouse e touchpad pelo libinput (toque para clicar ligado) e desenha o cursor do tema `XCURSOR_THEME`. `Ctrl+Alt+F1…F12` troca de terminal virtual.

Para testar dentro de outra sessão Wayland, o `sparky` abre numa janela:

```bash
cargo run -p sparky-compositor -- --spawn kitty
```

`--spawn` pode se repetir e abre cada comando já dentro do sparky. O layout do teclado vem do `/etc/vconsole.conf` ou, no Debian, do `/etc/default/keyboard`.

## Configuração

Tudo fica em `~/.config/sparky/sparky.toml`; sem o arquivo valem os padrões do sparky, que estão por extenso em `data/config/sparky.toml`. O sparky confere o arquivo a cada segundo e aplica a mudança sem reiniciar (`sparky-msg reload` força na hora); um arquivo inválido é recusado e a configuração anterior continua valendo.

```toml
autostart = ["nm-applet"]

[general]
gaps_inner = 5
gaps_outer = 10
border = 2
default_binds = true

[input]
layout = "br"
variant = "abnt2"
repeat_rate = 40
repeat_delay = 250

[apps]
terminal = "kitty"

[[bind]]
keys = "SUPER + B"
action = "spawn"
command = "firefox"

[[bind]]
keys = "SUPER + H"
action = "focus"
direction = "left"
```

Um `[[bind]]` com a mesma combinação de um atalho padrão substitui o padrão; `default_binds = false` descarta todos os padrões. Ações: `spawn` (`command`), `close-window`, `toggle-floating`, `fullscreen` (`mode` = `monitor` ou `maximized`), `toggle-split`, `focus` e `swap` (`direction` = `left`, `right`, `up`, `down`), `resize` (`width`, `height`), `workspace` (`workspace`), `move-to-workspace` (`workspace`, `follow`), `relative-workspace` (`step`), `previous-workspace`, `cycle-monitor` (`step`), `exit`. Sem `layout` em `[input]`, vale o teclado do sistema. Sem `terminal` em `[apps]`, vale `$TERMINAL` ou o primeiro terminal instalado.

## Sessão

O gerenciador de login (SDDM, GDM…) lista a sessão **Sparky** pelo `data/session/sparky.desktop`, instalado em `/usr/share/wayland-sessions`, que abre o `sparky` com `XDG_CURRENT_DESKTOP=sparky`. Ao subir direto no hardware, o sparky:

- exporta `WAYLAND_DISPLAY`, `DISPLAY`, `XDG_CURRENT_DESKTOP`, `XDG_SESSION_TYPE` e `SPARKY_SOCKET` para o D-Bus e o systemd (`dbus-update-activation-environment --systemd`), para os portais e os serviços ativados acharem a sessão;
- abre a barra (`sparky-bar`), as notificações (`mako`) e o primeiro agente polkit instalado;
- abre o autostart XDG (`~/.config/autostart` e `/etc/xdg/autostart`), respeitando `Hidden`, `OnlyShowIn`, `NotShowIn`, `TryExec` e `X-GNOME-Autostart-enabled`; um arquivo do usuário substitui o do sistema com o mesmo nome.

Aninhado em outra sessão, nada disso acontece. Os portais usam `data/portals/sparky-portals.conf` (GTK, e gnome-keyring para senhas), instalado em `/usr/share/xdg-desktop-portal`.

## Barra

Uma barra de 26 px no topo de cada monitor, como a do Omarchy: as áreas de trabalho à esquerda (de 1 a 5 sempre, as outras até a 10 quando existem; a ativa vira um ponto, as vazias ficam apagadas, a 10 aparece como 0), o relógio no centro e a bandeja à direita. Clicar numa área vai para ela; o botão direito no relógio alterna entre `quarta 09:05` e `quarta, 07 de outubro de 2026`. A barra fala com o sparky pelo IPC e se atualiza a cada mudança.

A bandeja usa StatusNotifierItem pelo D-Bus. O clique esquerdo ativa o aplicativo ou abre o menu quando o item anuncia apenas um menu; o direito abre o menu. Escolher uma entrada habilitada ou pressionar Esc fecha o menu. A superfície também fecha quando o item desaparece ou o menu fica vazio.

## IPC

O sparky abre um socket em `$XDG_RUNTIME_DIR/sparky-<WAYLAND_DISPLAY>.sock` e exporta o caminho em `SPARKY_SOCKET` para os programas que abre. Cada pedido e cada resposta é uma linha de JSON.

```bash
sparky-msg workspaces          # lista as áreas de trabalho
sparky-msg focus-workspace 3   # vai para a área 3
sparky-msg subscribe           # avisa a cada mudança nas áreas
sparky-msg reload              # relê o sparky.toml
sparky-msg capture output tela.png                 # o monitor em foco
sparky-msg capture window janela.png               # a janela em foco, sem a borda
sparky-msg capture region 0 0 800 600 regiao.png   # região relativa ao monitor em foco
```

## Captura de tela

O próprio sparky desenha a área pedida numa textura fora da tela (as mesmas janelas e camadas da tela, sem o cursor) e grava o PNG. O pedido chega pelo IPC (`capture`), com o monitor inteiro, a janela em foco ou uma região relativa a um monitor; uma região que sai da tela é cortada na borda.

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
