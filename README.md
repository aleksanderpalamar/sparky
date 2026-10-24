# Sparky

Ambiente de desktop Wayland escrito em Rust, com compositor próprio, feito para ser usado pelo teclado. O mouse funciona, mas tudo tem atalho.

Somente os temas vem do [Omarchy](https://github.com/basecamp/omarchy): como paletas de cores e os wallpapers.

## Organização

| Caminho | O que é |
|---|---|
| `crates/sparky-bar` | A barra do topo (`sparky-bar`): áreas de trabalho, relógio e bandeja, uma por monitor |
| `crates/sparky-compositor` | O compositor Wayland (binário `sparky`), feito com a biblioteca Smithay: protocolos, janelas, bordas, teclado e ponteiro |
| `crates/sparky-tiling` | O mosaico: árvore dwindle, folgas, foco por direção, flutuantes, tela cheia, áreas de trabalho e monitores. Não depende de Wayland |
| `crates/sparky-config` | O `sparky.toml`: esquema, padrões do sparky, ações e a tabela de atalhos padrão |
| `crates/sparky-ipc` | Protocolo do IPC do sparky (JSON por linha num socket Unix), o cliente usado pela barra e pelos aplicativos, e o `sparky-msg` |
| `crates/sparky-shot` | O capturador de tela (`sparky-shot`): seletor de modo, seleção de região pelo teclado ou mouse, PNG salvo e copiado |
| `crates/sparky-files` | O gerenciador de arquivos (`sparky-files`): grade e lista com ícones, dispositivos, lugares, filtro, copiar, recortar, colar, renomear, criar e lixeira |
| `crates/sparky-storage` | Volumes de bloco pelo udisksctl e smartphones MTP pelo udev/GIO/GVfs; parser de discos compartilhado com o painel |
| `crates/sparky-xdg` | Convenções XDG compartilhadas: pastas do usuário, leitura de `.desktop`, busca de programas e resolução de ícones dos temas |
| `crates/sparky-settings` | O painel de configurações (`sparky-settings`): edita o `sparky.toml` e aplica na hora pelo IPC |
| `crates/sparky-ui` | Peças comuns dos aplicativos iced: teclas, prompt, estilos, lateral numerada, rolagem e janela |
| `crates/sparky-launcher` | O lançador de aplicativos (`sparky-launcher`): busca nos `.desktop` instalados e abre pelo teclado |
| `crates/sparky-terminal` | O terminal padrão (`sparky-terminal`): abre o shell do usuário com as cores, a fonte e o layout do tema do sparky |
| `crates/sparky-keyring` | O chaveiro (`sparky-keyring`): guarda as senhas dos aplicativos num cofre cifrado e atende a API Secret Service do D-Bus |
| `crates/sparky-keyring-prompt` | A janela que pede a senha do cofre (`sparky-keyring-prompt`) |
| `crates/polkit-sparky` | Agente de autenticação do sistema, com janela nativa, temas e integração polkit/PAM |
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

Um `[[bind]]` com a mesma combinação de um atalho padrão substitui o padrão; `default_binds = false` descarta todos os padrões. Ações: `spawn` (`command`), `close-window`, `toggle-floating`, `fullscreen` (`mode` = `monitor` ou `maximized`), `toggle-split`, `focus` e `swap` (`direction` = `left`, `right`, `up`, `down`), `resize` (`width`, `height`), `workspace` (`workspace`), `move-to-workspace` (`workspace`, `follow`), `relative-workspace` (`step`), `previous-workspace`, `cycle-monitor` (`step`), `exit`. Cada `[[output]]` dá a escala de um monitor pelo nome (`sparky-msg outputs` lista os nomes), de 0.5 a 4, inclusive fracionada como 1.25 ou 1.5; sem regra vale 1. Os aplicativos recebem a escala pelo fractional-scale-v1 e desenham nítidos nela, e ao mudar a escala os monitores se reposicionam lado a lado. Sem `layout` em `[input]`, vale o teclado do sistema. Sem `terminal` em `[apps]`, vale `$TERMINAL` ou o primeiro terminal instalado, começando pelo `sparky-terminal`. `wallpaper` em `[general]` é o caminho da imagem de fundo (`~/` vale a pasta pessoal); `wallpaper = ""` deixa só a cor do tema, e sem a chave vale o primeiro papel de parede do Tokyo Night, se o pacote `sparky-wallpapers` estiver instalado.

## Sessão

O gerenciador de login (SDDM, GDM…) lista a sessão **Sparky** pelo `data/session/sparky.desktop`, instalado em `/usr/share/wayland-sessions`, que abre o `sparky` com `XDG_CURRENT_DESKTOP=sparky`. Ao subir direto no hardware, o sparky:

- exporta `WAYLAND_DISPLAY`, `DISPLAY`, `XDG_CURRENT_DESKTOP`, `XDG_SESSION_TYPE` e `SPARKY_SOCKET` para o D-Bus e o systemd (`dbus-update-activation-environment --systemd`), para os portais e os serviços ativados acharem a sessão;
- abre o chaveiro (`sparky-keyring`), o agente de autenticação (`polkit-sparky`), a barra (`sparky-bar`) e as notificações (`mako`);
- abre o autostart XDG (`~/.config/autostart` e `/etc/xdg/autostart`), respeitando `Hidden`, `OnlyShowIn`, `NotShowIn`, `TryExec` e `X-GNOME-Autostart-enabled`; um arquivo do usuário substitui o do sistema com o mesmo nome.

Aninhado em outra sessão, nada disso acontece. Os portais usam `data/portals/sparky-portals.conf` (GTK, wlr para a tela e `sparky-keyring` para senhas), instalado em `/usr/share/xdg-desktop-portal`.

O `polkit-sparky` apresenta pedidos de autorização do sistema, como montar partições internas. A janela é própria do Sparky: usa a paleta ativa, a fonte e os cantos retos dos aplicativos nativos, com campo de senha, seleção dos usuários permitidos pelo sistema, Enter para confirmar, Tab para mudar o controle e Esc para cancelar. A paleta acompanha as mudanças de tema enquanto a janela está aberta.

A autenticação usa o polkit e o PAM do sistema por meio da biblioteca `libpolkit-agent-1`; os pacotes não dependem de outro agente gráfico. A senha solicitada é a da conta indicada pelo polkit e é independente da senha do cofre `sparky-keyring`. O agente não guarda essa senha. Cookies de autenticação são enviados ao processo da janela por um pipe privado, sem argumentos de linha de comando. Pedidos são serializados e cancelados quando a autoridade solicita ou a sessão termina. O registro é limitado à sessão logind atual. A inicialização automática ocorre no próximo login no Sparky.

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
sparky-msg session lock        # o mesmo que as opções do menu do sistema:
sparky-msg session suspend     #   lock, suspend, logout, reboot e poweroff
```

## Captura de tela

O próprio sparky desenha a área pedida numa textura fora da tela (as mesmas janelas e camadas da tela, sem o cursor) e grava o PNG. O pedido chega pelo IPC (`capture`), com o monitor inteiro, a janela em foco ou uma região relativa a um monitor; uma região que sai da tela é cortada na borda.

Programas de terceiros capturam monitor ou janela individual pelos protocolos `ext-image-copy-capture-v1` e `ext-image-capture-source-v1`, com lista de janelas por `ext-foreign-toplevel-list-v1`. O compositor desenha diretamente em DMA-BUF na GPU, e o `xdg-desktop-portal-wlr` leva a imagem ao OBS e aos navegadores pelo PipeWire. O seletor nativo respeita o tema do Sparky. Os pacotes incluem os portais e o PipeWire como dependências. Captura de janela exige o backend wlr 0.8.0 ou posterior; recomenda-se 0.8.4 ou posterior. O protocolo `wlr-screencopy-unstable-v1` continua compatível com `grim` e backends antigos, com SHM para screenshots e DMA-BUF para vídeo. Veja [configuração, arquitetura e validação de desempenho](docs/SCREENCAST.md).

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

### Menu do sistema

`SUPER + ESC` abre o `sparky-launcher --system` (o `system_menu` de `[apps]`), com as mesmas teclas do lançador e cinco opções nesta ordem:

| Opção | O que faz |
|---|---|
| Bloquear | Abre o locker de `[apps]`, se a tela ainda não estiver bloqueada |
| Suspender | Bloqueia e suspende (`systemctl suspend`) assim que a tela trancar; se o locker não trancar em 5 segundos, suspende mesmo assim |
| Sair da sessão | Pede para todas as janelas fecharem e encerra o sparky quando a última fechar |
| Reiniciar | Fecha as janelas do mesmo jeito e roda `systemctl reboot` |
| Desligar | Fecha as janelas do mesmo jeito e roda `systemctl poweroff` |

Sair, Reiniciar e Desligar esperam os aplicativos fecharem sozinhos, como o hyprshutdown: nenhum é encerrado à força. Se depois de 8 segundos alguma janela continuar aberta (um aplicativo perguntando se quer salvar, por exemplo), o pedido é cancelado e uma notificação avisa quantas faltam; `SUPER + SHIFT + ESC` continua saindo na hora. Com o sparky aninhado em outra sessão, Suspender, Reiniciar e Desligar são recusados, para um teste nunca desligar a máquina; Bloquear e Sair funcionam.

## Terminal

O `sparky-terminal` é o terminal padrão: o sparky o procura antes dos outros terminais instalados, então ele abre no `SUPER + ENTER` e nos aplicativos de terminal do lançador. Ele abre o zsh com a configuração do próprio sparky (`ZDOTDIR=/usr/share/sparky/zsh`), sem ler o `~/.zshrc`, o oh-my-zsh ou o powerlevel10k do usuário; numa máquina nova ele já sai pronto. Para personalizar, copie a pasta para `~/.config/sparky/zsh/` (respeitando `XDG_CONFIG_HOME`): com um `.zshrc` lá, ela substitui a do sistema. Sem zsh instalado, abre o shell da conta. O shell recebe `TERM=xterm-256color`, `COLORTERM=truecolor` e `TERM_PROGRAM=sparky-terminal`.

A configuração (`data/zsh/`) reproduz a do powerlevel10k *rainbow* em zsh puro, nas cores do tema:

- prompt em duas linhas com moldura `╭─`/`╰─`, a pasta em azul, o git em verde (limpo) ou amarelo (com mudanças) com ramo, `⇣⇡`, `*` stash, `~` conflitos, `+` preparados, `!` alterados e `?` novos, o pontilhado até a direita, `✔` ou `✘ código`, o tempo do comando a partir de 3 segundos, tarefas em segundo plano e a hora; depois de executar, o prompt anterior encolhe para `❯`;
- histórico de 50 mil linhas em `~/.local/state/sparky/zsh_history`, completação com menu e sem diferenciar maiúsculas, setas buscando pelo começo da linha, `CTRL + ←/→` por palavra;
- aliases do git (`gst`, `gco`, `gcb`, `gaa`, `gcmsg`, `gp`, `gpsup`, `glog` e outros no padrão do oh-my-zsh), `copyfile`, `CTRL + O` copiando a linha (copybuffer), `ALT + ←/→/↑` voltando, avançando e subindo entre pastas (dirhistory), `google`, `ddg`, `github`, `youtube`, `wiki` e `stackoverflow` (web-search), e o autojump e o fzf quando instalados;
- `zsh-autosuggestions` e `zsh-syntax-highlighting` dos pacotes da distribuição, que o sparky instala junto.

A emulação (PTY, sequências VT/xterm, histórico de 10 mil linhas e seleção) vem da biblioteca `alacritty_terminal`; a janela, o desenho das células, o teclado e as cores são do sparky. O texto usa a JetBrainsMono Nerd Font em 12 px, com a margem de 18 px e os cantos retos do sparky, e mede a célula com o mesmo motor de texto que desenha, para as colunas não desalinharem. Negrito, itálico, sublinhado, riscado, inverso, esmaecido, 256 cores, truecolor, caracteres largos e os glifos da Nerd Font aparecem como no kitty.

As cores seguem o tema do sparky, inclusive com o terminal aberto: as 16 cores ANSI saem do `colors.toml` na mesma ordem que o Omarchy usa no kitty (`background`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `foreground`, `muted` e as `bright_*`), o cursor e a seleção usam `bright_foreground` e `selection`, e aplicar outro tema no painel recolore o terminal em até um segundo. Cores trocadas por um programa (OSC 4, 10, 11 e 12) valem até o programa desfazer.

| Tecla | O que faz |
|---|---|
| `CTRL + SHIFT + C` ou `CTRL + INSERT` | Copia a seleção |
| `CTRL + SHIFT + V` ou `SHIFT + INSERT` | Cola, entre as marcas de bracketed paste quando o programa pede, sem deixar o texto colado fechar o bloco |
| `SHIFT + PGUP` / `SHIFT + PGDN` | Rola o histórico uma página |
| `SHIFT + HOME` / `SHIFT + END` | Vai para o começo ou o fim do histórico |
| `SHIFT + ENTER` / `ALT + SHIFT + ENTER` | Envia `\e[13;2u` / `\e[13;4u`, como no kitty |

O mouse seleciona arrastando com o botão esquerdo; a roda rola o histórico, e em tela alternativa (`less`, `man`) vira setas. Digitar volta ao fim do histórico e limpa a seleção. O título da janela vem do shell e a janela fecha quando o shell termina.

```bash
sparky-terminal                       # o shell do usuário
sparky-terminal -d ~/Projetos         # começando nessa pasta
sparky-terminal -e htop               # roda um comando; o resto da linha são os argumentos dele
```

Abas e divisões ficam com o mosaico e os grupos do sparky. O terminal ainda não repassa o mouse para programas que pedem (vim, htop) nem implementa o protocolo de teclado do kitty.

## Chaveiro

O `sparky-keyring` guarda as senhas e tokens que os aplicativos pedem para lembrar (navegadores, VS Code, git com `libsecret`, clientes de e-mail). Ele atende o nome `org.freedesktop.secrets` no D-Bus da sessão com a API Secret Service, a mesma do gnome-keyring, então os aplicativos funcionam sem configuração. Para os aplicativos Flatpak, atende também o portal de segredos (`data/portals/sparky-keyring.portal`).

- O cofre fica em `~/.local/share/sparky/keyrings/login.json` (respeitando `XDG_DATA_HOME`), com a pasta só para o dono (700) e o arquivo também (600). Cada gravação substitui o arquivo de uma vez, sem deixar o cofre pela metade.
- O conteúdo é cifrado com XChaCha20-Poly1305 por uma chave derivada da senha pelo Argon2id (64 MiB, 3 passagens). Rótulos, atributos e segredos só existem cifrados; para a busca funcionar com o cofre trancado, o arquivo guarda um resumo SHA-256 de cada atributo, protegido contra alteração junto com o conteúdo.
- A senha do cofre é própria, separada da senha de login. Na primeira vez que um aplicativo quer guardar algo, o `sparky-keyring-prompt` pede para criar a senha (duas vezes, para conferir); depois, na primeira vez em cada sessão que um aplicativo precisa de uma senha guardada, pede para destrancar. Senha errada pede de novo; `Esc` cancela e o aplicativo recebe a recusa.
- O cofre fica destrancado até a sessão acabar: quando o compositor fecha, o `sparky-keyring` encerra e a chave some da memória.
- Há um cofre só, o padrão (`default`/`login`). O transporte das senhas pelo D-Bus usa o algoritmo `plain` da especificação; aplicativos que pedem o algoritmo cifrado recebem a recusa e repetem com `plain`, como manda a especificação.

Se outro chaveiro (gnome-keyring, KWallet) já tiver o nome `org.freedesktop.secrets` quando a sessão subir, o `sparky-keyring` avisa no log e não sobe; as senhas continuam com o outro chaveiro. As senhas que já estavam no gnome-keyring não são copiadas: cada aplicativo pede login uma vez dentro do sparky.

## Arquivos

O `sparky-files` abre pelo `SUPER + SHIFT + F` (ou `sparky-files <pasta>`). Abre em grade, com ícones e nomes de pastas e arquivos, e permite alternar para a lista com tamanho e data pelos botões Grade/Lista ou por `CTRL + 1`/`CTRL + 2`. A grade se adapta à largura da janela; as pastas vêm primeiro, e a visualização se atualiza quando a pasta muda. Trocar a visualização preserva filtro, seleção e itens marcados.

Os ícones seguem o tema configurado pelo painel no GTK, incluindo sua herança (`index.theme`). Pastas especiais e tipos comuns de arquivo têm ícones próprios; sem um ícone instalado, o sparky desenha um ícone de reserva. Uma troca de tema atualiza os ícones sem fechar o gerenciador.

Na lateral ficam os lugares e os dispositivos: SSDs, partições e unidades removíveis com sistema de arquivos, montados ou não. A lista se atualiza a cada dois segundos. Um clique abre o volume montado; se estiver desmontado, pede a montagem pelo `udisksctl` e abre o ponto retornado. O botão de desmontar é separado e nunca força a operação. `ALT + 1` a `9` montam/abrem os dispositivos correspondentes; `ALT + U` pede a desmontagem do volume aberto. A montagem depende do `udisks2` e das permissões do sistema; recusas e dispositivos ocupados aparecem como aviso. As consultas usam `lsblk` (`util-linux`). Partições do sistema, swap e contêineres criptografados ainda trancados não são oferecidos para montagem/desmontagem. Ao retirar o volume em que está navegando, o gerenciador volta à pasta pessoal.

Smartphones Android em modo de transferência de arquivos também aparecem na lateral, com ícone de telefone e indicação MTP. A descoberta usa os registros USB do udev e não monta automaticamente. Clicar conecta pelo GIO/GVfs, reutilizando um acesso existente e abrindo seu caminho FUSE dentro do próprio sparky-files. O botão de desmontar encerra esse acesso sem forçar. O telefone deve estar desbloqueado e autorizar a transferência de arquivos; dispositivos em modo de carregamento e iPhones não estão cobertos por esse suporte MTP.

O backend depende de `gvfs-mtp` no Arch e Fedora; no Debian/Ubuntu, de `gvfs-backends` e `gvfs-fuse`. No Fedora também é necessário `gvfs-fuse`; no Arch o acesso FUSE já faz parte de `gvfs`. Os pacotes declaram esses componentes como dependências opcionais/recomendadas, mantendo a política usada para o udisks2. Não é necessário executar outro gerenciador de arquivos. Falhas de acesso, backend ausente e telefone ocupado aparecem como aviso, e a espera das operações assíncronas de montagem/desmontagem é limitada a 30 segundos.

Quando o polkit exige autenticação, o agente gráfico da sessão pede a senha. O gerenciador continua respondendo enquanto aguarda e permite tentar novamente após cancelar. O `udisksctl` é executado sem o terminal de controle da sessão, pelo `setsid` do `util-linux`, para evitar pedidos de senha invisíveis no terminal. Os tooltips têm fundo opaco com as cores do tema.

As consultas periódicas de dispositivos não exibem indicador de atualização. A lateral mostra progresso apenas durante montagem e desmontagem, mantém os avisos de erro e preserva a mensagem de lista vazia durante as consultas.

| Tecla | Ação |
|---|---|
| `j` `k` ou `↑` `↓` | Desce e sobe uma linha da grade ou lista |
| `←` `→` | Na grade, escolhe o item vizinho; na lista, volta/abre |
| `Enter` ou `l` | Abre a pasta ou o arquivo (no aplicativo padrão, pelo `xdg-open`) |
| `h`, `Backspace` ou `ALT + ↑` | Volta para a pasta de cima, com a de origem selecionada |
| `g` / `G`, `Home` / `End`, `PageUp` / `PageDown` | Começo, fim e páginas |
| `1` a `9` | Vai para o lugar com esse número |
| `/` | Filtra pelo nome enquanto digita; `Enter` fixa, `Esc` limpa |
| `.` ou `CTRL + H` | Mostra ou esconde os ocultos |
| `Espaço` | Marca o item e avança; as ações valem para os marcados ou, sem marcas, para o selecionado |
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
| Aparência | Folgas, espessura da borda, tamanho das janelas flutuantes, os atalhos padrão e o papel de parede |
| Teclado | Layout, variante e opções do XKB (conferidos com o `evdev.lst` antes de gravar) e a repetição das teclas |
| Aplicativos | Terminal, lançador, arquivos, configurações, capturador e bloqueio de tela que os atalhos abrem |
| Inicialização | Comandos rodados quando a sessão começa |
| Atalhos | Lista dos atalhos em uso, com o que cada um faz |
| Monitores | Monitores que o sparky está usando |
| Energia | Perfil de energia (`powerprofilesctl`), brilho da tela (`brightnessctl`) e bateria (`upower`) |
| Data e hora | Fuso horário, hora local, sincronização pela rede e relógio do hardware (`timedatectl`) |
| Região e idioma | Idioma, teclado do console e layout padrão do sistema (`localectl`) |
| Usuários | Consulta de contas, identificadores, grupos e sessões (`getent` e `loginctl`) |
| Mídia | Volumes externos, montagem e desmontagem (`lsblk` e `udisksctl`) |
| Temas | Catálogo e aplicação de paletas nativas do Sparky |
| Sobre | Versão do sparky, sistema, kernel e onde fica a configuração |

`Tab` e `SHIFT + Tab` (ou `1` a `9`) trocam de seção, `j`/`k` escolhem o item, `h`/`l` diminuem e aumentam números ou ligam e desligam, `Enter` edita um texto ou um número, `d` remove um comando da inicialização, `Esc` desiste da edição e `q` fecha. O mouse também funciona: `−`/`+`, clique no valor para editar e clique duplo na inicialização.

Em Data e hora, `Enter` ou um clique no fuso abre a edição; o nome precisa existir na lista de `timedatectl list-timezones`. A hora local é somente leitura. `Enter` ou `h`/`l` alternam a hora pela rede e o uso de hora local no relógio do hardware; sem serviço NTP disponível, a linha da rede é somente leitura. `F5` atualiza os dados. Alterações dependem das permissões do sistema, e recusas aparecem no aviso do painel.

Em Região e idioma, `h`/`l` percorrem as opções instaladas e `Enter` ou um clique permite digitar um valor da lista do `localectl`. Sem opções disponíveis, a linha é somente leitura. Esses valores são padrões do sistema; o teclado da sessão Sparky continua sendo configurado na seção Teclado. Mudanças de idioma podem exigir uma nova sessão para os aplicativos adotarem o valor.

Usuários mostra uma conta por vez, navegável por `j`/`k` ou pelos botões laterais: nome, UID, tipo, sessão, pasta pessoal, interpretador e grupos. Reconhece administradores nos grupos `wheel` e `sudo`; se uma consulta falhar, o dado aparece como indisponível. Assim como no painel de exemplo, esta seção é somente leitura: não cria contas nem altera senhas ou grupos. `F5` atualiza a consulta. Seções após a nona continuam acessíveis por `Tab`, `SHIFT + Tab` e clique na lateral.

Mídia lista sistemas de arquivos de discos externos/removíveis e volumes em `/run/media` ou `/media`, com rótulo, tamanho e ponto de montagem. `Enter` ou clique alterna a montagem; `l` monta, `h` desmonta e `F5` relê a lista. A montagem usa `udisksctl` (pacote `udisks2`) e as permissões do sistema. Swap, partições sem sistema de arquivos e montagens do sistema não são oferecidas.

## Tema

As cores saem do primeiro arquivo que existir:

1. `~/.config/sparky/colors.toml`
2. `$SPARKY_PATH/themes/tokyo-night/colors.toml` (`SPARKY_PATH` é `/usr/share/sparky` quando instalado)

Sem nenhum dos dois, vale o Tokyo Night embutido.

A seção Temas lista paletas em `$XDG_DATA_HOME/sparky/themes/<nome>/colors.toml` (por padrão `~/.local/share/sparky/themes`) e em `$SPARKY_PATH/themes` (por padrão `/usr/share/sparky/themes`). O usuário tem prioridade sobre nomes iguais do sistema; Tokyo Night permanece disponível. Arquivos sem cores reconhecidas ou com cores inválidas são ignorados.

O sparky traz 22 temas: catppuccin, catppuccin-latte, ethereal, everforest, flexoki-light, gruvbox, hackerman, kanagawa, last-horizon, lumon, lupine, matte-black, miasma, nord, osaka-jade, retro-82, ristretto, rose-pine, solitude, tokyo-night, vantablack e white. Além do `colors.toml`, cada tema pode ter um `icons.theme` com o nome do tema de ícones que combina com ele (os temas do sparky usam as cores do Yaru, como `Yaru-blue` e `Yaru-sage`).

`j`/`k` escolhem a paleta; `Enter` ou clique aplica. As cores são gravadas por substituição atômica em `~/.config/sparky/colors.toml` (respeitando `XDG_CONFIG_HOME`). Se o tema tiver `icons.theme`, o painel também troca o tema de ícones: grava `gtk-icon-theme-name` no `settings.ini` do GTK 3 e do GTK 4 (as outras chaves ficam) e roda `gsettings set org.gnome.desktop.interface icon-theme`. Esse ajuste é do GTK, então vale também fora do sparky. O painel atualiza imediatamente; compositor, barra, lançador, arquivos e capturador verificam as cores a cada segundo, preservando janelas, seleção e edições abertas. A seção não depende de Hyprland ou hypr-theme e não gerencia temas de aplicativos externos.

### Papel de parede

O compositor desenha o papel de parede atrás de tudo em cada monitor, recortado para cobrir a tela sem distorcer, no tamanho em pixels do monitor (a escala conta). A imagem (JPEG, PNG ou WebP) é carregada fora do laço do compositor; até ficar pronta, vale a cor de fundo do tema. Uma mudança no `sparky.toml` troca o papel de parede na hora.

Os papéis de parede de cada tema ficam em `$XDG_DATA_HOME/sparky/wallpapers/<tema>/` (os do usuário vêm primeiro) e em `$SPARKY_PATH/wallpapers/<tema>/`, onde o pacote `sparky-wallpapers` instala 67 imagens dos 22 temas, em ordem de nome. Aplicar um tema no painel escolhe o primeiro papel de parede dele. A linha **Papel de parede**, no topo da seção Temas, mostra qual está em uso (`2 de 3 · 2-b.webp`) e `h`/`l` ou `Enter` passam para o anterior ou o próximo; a escolha vai para `wallpaper` no `sparky.toml`. Em Aparência dá para digitar qualquer caminho.

### Fontes

A barra, as abas dos grupos e os aplicativos do sparky usam a JetBrainsMono Nerd Font; sem ela instalada, o texto cai numa fonte do sistema. As fontes padrão do resto do sistema não mudam. Os pacotes recomendam as fontes que acompanham os temas: JetBrainsMono Nerd Font, Liberation, Noto (com emoji e CJK) e Font Awesome. No Debian e no Fedora não há pacote da variante Nerd; o recomendado é a JetBrains Mono comum, e a Nerd pode ser instalada à parte.

## Pacotes

O `packaging/build.sh` gera os três formatos em `packaging/out/`:

```bash
packaging/build.sh           # .deb, .rpm e pacman
packaging/build.sh deb       # só um deles: deb, rpm ou pacman
```

- `.deb` e `.rpm` são compilados com `cargo zigbuild` contra a glibc 2.36, para rodar no Debian 12+, Ubuntu 24.04+ e Fedora 40+; o script para se algum binário pedir uma glibc mais nova. Precisam de `zig`, `cargo-zigbuild`, `cargo-deb` e `cargo-generate-rpm`.
- O pacote do pacman sai do `packaging/arch/PKGBUILD`, que compila o que está commitado no repositório com `makepkg`.
- Os papéis de parede saem num pacote à parte, `sparky-wallpapers` (variante `wallpapers` do `cargo-deb` e do `cargo-generate-rpm`, e pacote dividido no PKGBUILD), para o `sparky` continuar leve.

Os três instalam os binários em `/usr/bin`, a sessão em `/usr/share/wayland-sessions` (aparece no SDDM e no GDM), os temas e o `sparky.toml` de exemplo em `/usr/share/sparky`, as entradas dos aplicativos e o `sparky-mimeapps.list`, que faz as pastas abrirem no `sparky-files` dentro da sessão.

O [workflow de Releases](ci/RELEASE.md) gera os pacotes pelo GitHub Actions ao
enviar uma tag `vMAJOR.MINOR.PATCH`, como `v0.22.0`. A tag precisa corresponder
à versão dos manifests e apontar para um commit integrado na main. Depois do
Quality Gate e dos builds, os pacotes DEB, RPM e Arch, incluindo wallpapers,
são publicados na Release com hashes SHA-256 e o commit de origem.

`python3 packaging/test_build.py -v` testa o script de empacotamento com ferramentas simuladas, incluindo a localização do manifesto do RPM e a interrupção quando a compilação ou a checagem da glibc falha.

## Desenvolvimento

O compositor mantém logs rotativos e amostras de memória e do agendamento de
quadros em `~/.local/state/sparky/logs/`, respeitando `XDG_STATE_HOME`.
Veja o [diagnóstico do compositor](docs/DIAGNOSTICS.md) para interpretar os
contadores, a retenção e os limites da investigação de travamentos e vazamentos.

```bash
cargo test
cargo clippy --all-targets
```

Os PRs são validados pelo [Quality Gate](ci/README.md): convenções, checklist de
revisão, build e testes em debug/release, Clippy, documentação e regras dos três
formatos de pacote. O CI usa runners isolados e não inicia uma sessão gráfica.
Consulte o documento para reproduzir as verificações localmente e configurar o
check obrigatório na main.

## Créditos

Os temas de `data/themes` (paletas e nomes dos temas de ícones), os papéis de parede de `data/wallpapers` e o layout vêm do Omarchy, © David Heinemeier Hansson, licença MIT; a licença acompanha cada tema. O repositório do Omarchy não informa a autoria das imagens.

## Licença

GPL-3.0-or-later. O texto está em `LICENSE`.
