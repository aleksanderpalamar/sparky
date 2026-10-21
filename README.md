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
| `crates/sparky-shot` | O capturador de tela (`sparky-shot`): seletor de modo, seleção de região pelo teclado ou mouse, PNG salvo e copiado |
| `crates/sparky-files` | O gerenciador de arquivos (`sparky-files`): lista navegável pelo teclado, lugares, filtro, copiar, recortar, colar, renomear, criar e lixeira |
| `crates/sparky-xdg` | Convenções XDG compartilhadas: pastas do usuário (`user-dirs.dirs`), leitura de `.desktop` (nome no idioma, `OnlyShowIn`, `Exec`) e busca de programas no `PATH` |
| `crates/sparky-settings` | O painel de configurações (`sparky-settings`): edita o `sparky.toml` e aplica na hora pelo IPC |
| `crates/sparky-ui` | Peças comuns dos aplicativos iced: teclas, prompt, estilos, lateral numerada, rolagem e janela |
| `crates/sparky-launcher` | O lançador de aplicativos (`sparky-launcher`): busca nos `.desktop` instalados e abre pelo teclado |
| `crates/sparky-theme` | Paleta de cores compartilhada pelos aplicativos, lida do `colors.toml` do tema |
| `data/themes` | Temas no formato `colors.toml` |

## Mosaico

Cada janela nova divide a caixa da janela em foco: lado a lado se a caixa for mais larga que alta, empilhada se não, e a nova fica à direita ou embaixo. A divisão escolhida é preservada quando a caixa muda de tamanho e pode ser invertida à mão. Fechar uma janela devolve o espaço para a vizinha. Entre janelas a folga é de 5 px de cada lado, e de 10 px até a borda da tela (`Gaps::SPARKY`).

As áreas de trabalho vão de 1 a 10 e cada uma pertence a um monitor. Ir para uma área visível em outro monitor leva o foco até ele; uma área vazia que sai da tela deixa de existir.

O rascunho é uma área extra que aparece por cima da área atual, como a área especial do Hyprland: `SUPER + S` (ou `SUPER` + a tecla à esquerda do 1) mostra e esconde, e `SUPER + ALT + S` (ou com `SHIFT` na tecla à esquerda do 1) manda a janela em foco para ele. Com o rascunho aberto, as janelas novas abrem nele, o foco fica nele e as janelas de baixo não respondem ao mouse; trocar de área esconde o rascunho e ele abre no monitor em foco. As ações no `sparky.toml` são `toggle-scratchpad` e `move-to-scratchpad`.

Grupos juntam várias janelas num só quadro do mosaico, com uma barra de abas no topo mostrando o título de cada uma: `SUPER + G` agrupa a janela em foco (ou desfaz o grupo), `SUPER + ALT + G` tira a janela do grupo, `SUPER + ALT` + seta põe a janela no grupo vizinho, `SUPER + ALT + TAB` e `SUPER + CTRL` + seta andam pelas abas e `SUPER + ALT + 1` a `5` escolhem a aba. Uma janela aberta a partir de outra que está num grupo entra nele. As ações no `sparky.toml` são `toggle-group`, `leave-group`, `move-into-group` (com `direction`), `cycle-group` (com `step`) e `select-in-group` (com `index`, a partir de 1).

Com o mouse, como no Omarchy: `SUPER` + botão esquerdo arrasta uma janela flutuante, e numa janela do mosaico troca de lugar com a janela onde ela for solta; `SUPER` + botão direito redimensiona pelo canto mais perto do ponteiro (no mosaico, move a divisão). Uma janela não encolhe abaixo de 120 × 80. Os pedidos de mover e redimensionar feitos pelos próprios aplicativos (barra de título desenhada pelo cliente, Wayland ou X11) usam o mesmo caminho.

## Compositor

O `sparky` é o próprio compositor Wayland; não depende de nenhum outro. Fala xdg-shell (janelas), wlr-layer-shell (barra, lançador, notificações), decorações do lado do servidor, área de transferência (incluindo seleção primária e wlr-data-control), viewporter, cursor-shape (o aplicativo pede o cursor pelo nome: texto, mão, redimensionar), relative-pointer (movimento cru do mouse, para jogos e programas 3D) e pointer-constraints (o aplicativo pode travar o ponteiro no lugar ou prendê-lo dentro da janela) e ext-session-lock (tela de bloqueio).

Aplicativos X11 rodam pelo XWayland, que o sparky sobe sozinho e controla com o gerenciador de janelas X11 do Smithay: janelas X11 entram no mosaico como as Wayland, menus e dicas X11 (janelas sem gerência) ficam por cima de tudo, e a área de transferência e a seleção primária passam nos dois sentidos. Os programas abertos pelo sparky recebem `DISPLAY` e `WAYLAND_DISPLAY`.

`SUPER + CTRL + L` bloqueia a sessão com o locker de `[apps]` (`swaylock` por padrão; `hyprlock` e outros que falem ext-session-lock também servem). Bloqueado, só a tela do locker aparece e recebe teclado e mouse, os atalhos param (menos `Ctrl+Alt+F1…F12`) e a captura de tela também só vê o locker. Se o locker morrer sem desbloquear, a sessão continua bloqueada com a tela vazia, e um locker aberto de novo retoma o bloqueio.

Toda janela comum entra no mosaico. Flutuam as que têm janela-mãe (diálogos), as de tamanho fixo e as do painel de configurações e do capturador. A borda da janela em foco usa a cor de destaque do tema, e o foco do teclado acompanha o ponteiro.

Rodado num terminal virtual (ou pelo gerenciador de login), o `sparky` assume a placa de vídeo pela sessão do sistema (libseat), liga cada monitor conectado lado a lado da esquerda para a direita, acompanha monitor ligado e desligado em tempo real, lê teclado, mouse e touchpad pelo libinput (toque para clicar ligado) e desenha o cursor do tema `XCURSOR_THEME`, com o formato que cada aplicativo pedir. `Ctrl+Alt+F1…F12` troca de terminal virtual.

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

[[output]]
name = "DP-1"
scale = 1.5
```

Um `[[bind]]` com a mesma combinação de um atalho padrão substitui o padrão; `default_binds = false` descarta todos os padrões. Ações: `spawn` (`command`), `close-window`, `toggle-floating`, `fullscreen` (`mode` = `monitor` ou `maximized`), `toggle-split`, `focus` e `swap` (`direction` = `left`, `right`, `up`, `down`), `resize` (`width`, `height`), `workspace` (`workspace`), `move-to-workspace` (`workspace`, `follow`), `relative-workspace` (`step`), `previous-workspace`, `cycle-monitor` (`step`), `exit`. Cada `[[output]]` dá a escala de um monitor pelo nome (`sparky-msg outputs` lista os nomes), de 0.5 a 4, inclusive fracionada como 1.25 ou 1.5; sem regra vale 1. Os aplicativos recebem a escala pelo fractional-scale-v1 e desenham nítidos nela, e ao mudar a escala os monitores se reposicionam lado a lado. Sem `layout` em `[input]`, vale o teclado do sistema. Sem `terminal` em `[apps]`, vale `$TERMINAL` ou o primeiro terminal instalado.

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
sparky-msg outputs             # lista os monitores e qual está em foco
sparky-msg capture output tela.png                 # o monitor em foco
sparky-msg capture window janela.png               # a janela em foco, sem a borda
sparky-msg capture region 0 0 800 600 regiao.png   # região relativa ao monitor em foco
```

## Captura de tela

O próprio sparky desenha a área pedida numa textura fora da tela (as mesmas janelas e camadas da tela, sem o cursor) e grava o PNG. O pedido chega pelo IPC (`capture`), com o monitor inteiro, a janela em foco ou uma região relativa a um monitor; uma região que sai da tela é cortada na borda.

Programas de terceiros também capturam a tela pelo protocolo `wlr-screencopy-unstable-v1`: o `grim` (monitor inteiro ou `-g` com região) e o `xdg-desktop-portal-wlr`, que leva o compartilhamento de tela a navegadores e ao OBS. A imagem vai na orientação do monitor, sem o cursor.

O `sparky-shot` é o capturador que usa esse pedido:

| Atalho | Comando | O que faz |
|---|---|---|
| `PRINT` | `sparky-shot region` | Abre a seleção de região no monitor em foco |
| `SHIFT + PRINT` | `sparky-shot window` | Captura a janela em foco |
| `CTRL + PRINT` | `sparky-shot output` | Captura o monitor em foco |
| `SUPER + PRINT` | `sparky-shot` | Abre o seletor: `R` região, `J` janela, `M` monitor, `Tab` troca o atraso (nenhum, 3, 5 ou 10 s), `Esc` cancela |

Na seleção de região, as setas movem a área 10 px (1 px com `CTRL`), `SHIFT` com as setas redimensiona, `A` pega a tela toda, `Enter` captura e `Esc` cancela. Arrastar com o mouse desenha uma área nova. O atraso conta depois da seleção, para dar tempo de abrir um menu.

A imagem vai para `Capturas de tela` dentro da pasta de imagens (`XDG_PICTURES_DIR`, ou `~/Pictures`) com o nome `captura-AAAA-MM-DD_HH-MM-SS.png`, é copiada para a área de transferência pelo `wl-copy` e uma notificação avisa onde ficou (`notify-send`).

## Lançador

O `sparky-launcher` abre pelo `SUPER + ESPAÇO`: uma caixa no centro da tela com a busca já focada e a lista dos aplicativos instalados, com o nome no idioma do sistema. Digitar filtra (começo do nome primeiro, depois começo de palavra, trecho do nome, palavras-chave e descrição, e por fim as letras na ordem, como `ffx` para Firefox); setas, `Tab`/`SHIFT + Tab` ou `CTRL + N`/`P` (ou `J`/`K`) escolhem; `Enter` abre e `Esc` fecha. O clique também abre. A lista vem dos `.desktop` de `~/.local/share/applications` e das pastas de `XDG_DATA_DIRS` (Flatpak incluso), respeitando `NoDisplay`, `Hidden`, `OnlyShowIn`/`NotShowIn` e `TryExec`; aplicativos de terminal abrem no terminal escolhido, que o sparky passa aos programas em `TERMINAL`.

## Arquivos

O `sparky-files` abre pelo `SUPER + SHIFT + F` (ou `sparky-files <pasta>`). À esquerda ficam os lugares (pasta pessoal e as pastas do `user-dirs.dirs` que existem); no centro, a pasta atual com pastas primeiro, tamanho e data. A lista se atualiza sozinha quando a pasta muda.

| Tecla | Ação |
|---|---|
| `j` `k` ou setas | Desce e sobe |
| `Enter`, `l` ou `→` | Abre a pasta ou o arquivo (no aplicativo padrão, pelo `xdg-open`) |
| `h`, `Backspace`, `←` ou `ALT + ↑` | Volta para a pasta de cima, com a de origem selecionada |
| `g` / `G`, `Home` / `End`, `PageUp` / `PageDown` | Começo, fim e páginas |
| `1` a `9` | Vai para o lugar com esse número |
| `/` | Filtra pelo nome enquanto digita; `Enter` fixa, `Esc` limpa |
| `.` ou `CTRL + H` | Mostra ou esconde os ocultos |
| `Espaço` | Marca o item e desce; as ações valem para os marcados ou, sem marcas, para o selecionado |
| `y` / `x` / `p` (ou `CTRL + C` / `X` / `V`) | Copia, recorta e cola; colar na mesma pasta cria `nome (cópia).ext` |
| `r` ou `F2` | Renomeia, já com o nome sem a extensão selecionado |
| `a` | Cria: `nome/` vira pasta, sem a barra vira arquivo vazio |
| `d` ou `Delete` | Manda para a lixeira (`~/.local/share/Trash`, com o `.trashinfo`), depois de confirmar com `Enter` |
| `F5` ou `CTRL + R` | Relê a pasta |
| `q` ou `CTRL + Q` | Fecha |

Com o mouse, um clique seleciona, dois cliques abrem e a roda rola a lista.

## Configurações

O `sparky-settings` abre pelo `SUPER + ALT + ESPAÇO`. As mudanças de configuração do sparky são gravadas no `~/.config/sparky/sparky.toml` na hora, só na chave mexida (os comentários do arquivo ficam), e o sparky recarrega pelo IPC. Se o sparky não estiver rodando, a mudança vale quando ele iniciar. As seções de sistema executam as ferramentas correspondentes e mostram o resultado ou o erro, sem gravar no TOML.

| Seção | O que muda |
|---|---|
| Aparência | Folgas, espessura da borda, tamanho das janelas flutuantes e os atalhos padrão |
| Teclado | Layout, variante e opções do XKB (conferidos com o `evdev.lst` antes de gravar) e a repetição das teclas |
| Aplicativos | Terminal, lançador, arquivos, configurações, capturador e bloqueio de tela que os atalhos abrem |
| Inicialização | Comandos rodados quando a sessão começa |
| Atalhos | Lista dos atalhos em uso, com o que cada um faz |
| Monitores | Monitores que o sparky está usando |
| Energia | Perfil de energia (`powerprofilesctl`), brilho da tela (`brightnessctl`) e bateria (`upower`) |
| Data e hora | Fuso horário, hora local, sincronização pela rede e relógio do hardware (`timedatectl`) |
| Região e idioma | Idioma, teclado do console e layout padrão do sistema (`localectl`) |
| Sobre | Versão do sparky, sistema, kernel e onde fica a configuração |

`Tab` e `SHIFT + Tab` (ou `1` a `9`) trocam de seção, `j`/`k` escolhem o item, `h`/`l` diminuem e aumentam números ou ligam e desligam, `Enter` edita um texto ou um número, `d` remove um comando da inicialização, `Esc` desiste da edição e `q` fecha. O mouse também funciona: `−`/`+`, clique no valor para editar e clique duplo na inicialização.

Em Data e hora, `Enter` ou um clique no fuso abre a edição; o nome precisa existir na lista de `timedatectl list-timezones`. A hora local é somente leitura. `Enter` ou `h`/`l` alternam a hora pela rede e o uso de hora local no relógio do hardware; sem serviço NTP disponível, a linha da rede é somente leitura. `F5` atualiza os dados. Alterações dependem das permissões do sistema, e recusas aparecem no aviso do painel.

Em Região e idioma, `h`/`l` percorrem as opções instaladas e `Enter` ou um clique permite digitar um valor da lista do `localectl`. Sem opções disponíveis, a linha é somente leitura. Esses valores são padrões do sistema; o teclado da sessão Sparky continua sendo configurado na seção Teclado. Mudanças de idioma podem exigir uma nova sessão para os aplicativos adotarem o valor.

## Tema

As cores saem do primeiro arquivo que existir:

1. `~/.config/sparky/colors.toml`
2. `$SPARKY_PATH/themes/tokyo-night/colors.toml` (`SPARKY_PATH` é `/usr/share/sparky` quando instalado)

Sem nenhum dos dois, vale o Tokyo Night embutido.

## Pacotes

O `packaging/build.sh` gera os três formatos em `packaging/out/`:

```bash
packaging/build.sh           # .deb, .rpm e pacman
packaging/build.sh deb       # só um deles: deb, rpm ou pacman
```

- `.deb` e `.rpm` são compilados com `cargo zigbuild` contra a glibc 2.36, para rodar no Debian 12+, Ubuntu 24.04+ e Fedora 40+; o script para se algum binário pedir uma glibc mais nova. Precisam de `zig`, `cargo-zigbuild`, `cargo-deb` e `cargo-generate-rpm`.
- O pacote do pacman sai do `packaging/arch/PKGBUILD`, que compila o que está commitado no repositório com `makepkg`.

Os três instalam os binários em `/usr/bin`, a sessão em `/usr/share/wayland-sessions` (aparece no SDDM e no GDM), o tema e o `sparky.toml` de exemplo em `/usr/share/sparky`, as entradas dos aplicativos e o `sparky-mimeapps.list`, que faz as pastas abrirem no `sparky-files` dentro da sessão.

`python3 packaging/test_build.py -v` testa o script de empacotamento com ferramentas simuladas, incluindo a localização do manifesto do RPM e a interrupção quando a compilação ou a checagem da glibc falha.

## Desenvolvimento

```bash
cargo test
cargo clippy --all-targets
```

## Créditos

O tema Tokyo Night (`data/themes/tokyo-night/colors.toml`) e o layout vêm do Omarchy, © David Heinemeier Hansson, licença MIT.

## Licença

GPL-3.0-or-later. O texto está em `LICENSE`.
