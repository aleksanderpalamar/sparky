# Compartilhamento de tela e OBS

O Sparky oferece captura de monitor e janela individual pelo portal ScreenCast.
O seletor `sparky-shot --share-source` usa o layout e a paleta dos aplicativos
nativos. Escolha uma fonte e confirme com Enter ou clique; Esc cancela.

O caminho de vídeo é:

```text
Sparky/GLES → DMA-BUF → xdg-desktop-portal-wlr → PipeWire → OBS
```

O compositor desenha diretamente no DMA-BUF fornecido pelo cliente. Esse caminho
não chama `copy_framebuffer`, `map_texture` nem converte pixels em um vetor na
CPU. A sincronização da GPU é assíncrona: usa um descritor de fence no loop de
eventos ou consulta não bloqueante quando não há fence exportável. Não espera
pela GPU com `wait()` no loop do compositor.

Cada sessão nativa pede no máximo um quadro por vez. O compositor limita a
captura a 60 FPS e mantém no máximo oito quadros pendentes/em execução no total
e dezesseis sessões. Sem alteração na imagem, aguarda nova atualização; uma
janela de outro workspace recebe callbacks após submissões bem-sucedidas e
enquanto aguarda mudanças, para continuar produzindo quadros mesmo com outros
aplicativos atualizando a cena.
Fechar a fonte encerra a sessão, e redimensioná-la renegocia os buffers.

A captura de uma janela usa apenas sua árvore de superfícies e popups. Janelas
que a cobrem e outros aplicativos não entram nessa imagem. O cursor pode ser
oculto ou incorporado, conforme o pedido do portal. Cursor separado em metadados
não é oferecido. Uma sessão iniciada durante o bloqueio é recusada; capturas em
andamento mascaram a imagem enquanto a sessão está bloqueada. Um quadro cuja
entrega atravesse uma mudança desse estado é refeito com a máscara atual antes
de ser entregue, sem encerrar a transmissão nem publicar o conteúdo obsoleto.

O protocolo antigo `wlr-screencopy` continua disponível com buffers SHM para
screenshots, inclusive `grim`, e também aceita DMA-BUF para vídeo. O capturador
PNG mantém seu caminho de leitura de pixels. Os buffers antigos preservam a
transformação do monitor; o protocolo novo entrega pixels na orientação normal
com metadado explícito. No protocolo novo de vídeo, um
buffer incompatível falha explicitamente, sem voltar silenciosamente para
leitura de pixels na CPU.

## Dependências e instalação

Os pacotes passam a exigir `xdg-desktop-portal`, `xdg-desktop-portal-wlr`,
`xdg-desktop-portal-gtk` e `pipewire`. O backend wlr é compatível com os protocolos
Wayland oferecidos pelo Sparky; o compositor continua em Rust/Smithay.

A captura de janelas exige `xdg-desktop-portal-wlr` 0.8.0 ou posterior. Recomenda-se
0.8.4 ou posterior, que corrige falta de buffers durante a captura. O pacote Arch
exige essa versão. DEB/RPM permitem backends anteriores para preservar a
instalação nas distribuições suportadas; esses backends oferecem somente a
captura de monitor pelo protocolo antigo. O Debian 12 precisa de um backend
atualizado para oferecer também janelas.

O arquivo `/etc/xdg/xdg-desktop-portal-wlr/sparky` configura o seletor e o limite
de FPS. Uma configuração pessoal de `xdg-desktop-portal-wlr` pode ter prioridade.
Após instalar a atualização e as dependências, entre novamente na sessão do
Sparky para que os serviços de portal obtenham o ambiente e os novos protocolos.

No OBS, adicione **Captura de tela (PipeWire)** ou **Captura de janela
(PipeWire)**, conforme os nomes oferecidos pela instalação. A fonte solicitará
o seletor do Sparky. Fechar a fonte no aplicativo interrompe o compartilhamento.
Essa integração não instala o OBS nem altera suas configurações de gravação.

H.264 e HEVC são formatos de vídeo. A escolha do encoder no OBS é uma etapa
posterior à captura: usar DMA-BUF não seleciona automaticamente um codec nem
ativa VA-API. A negociação final de formatos/modificadores e o uso de encoding
por hardware dependem do OBS, do PipeWire e do driver.

## Validação e limites

Os testes automáticos usam sockets Wayland privados e janelas de teste sem GPU,
isolados do display e D-Bus da sessão real. Verificam anúncio de protocolos,
restrições dos buffers, seleção de janela, redimensionamento, remoção de fontes,
cancelamento, limites, regras de agendamento e instalação dos dados/dependências.
Não gravam a tela do usuário nem iniciam o OBS.

A validação real ainda deve conferir monitor, janela do ETS2, oclusão, troca de
workspace, cursor, bloqueio, escalas, rotação e redimensionamento. Confirmar a
importação do DMA-BUF no OBS é necessário para afirmar que todo o caminho ficou
na GPU. Um build verde não comprova essa negociação.

Para avaliar desempenho, compare o mesmo trecho do jogo sem captura e com
captura em 1080p60, usando as mesmas configurações. Observe FPS/frametime do
jogo, quadros perdidos por renderização/codificação nas Estatísticas do OBS e
RSS/VRAM do compositor durante uma gravação prolongada e após interrompê-la.
Não há garantia de custo zero: a composição adicional e a codificação consomem
recursos da GPU. Esse teste deve ser feito com autorização na sessão real.

Referências: [protocolos Wayland](https://gitlab.freedesktop.org/wayland/wayland-protocols),
[releases do backend](https://github.com/emersion/xdg-desktop-portal-wlr/releases),
[configuração e contrato do seletor](https://github.com/emersion/xdg-desktop-portal-wlr/blob/master/xdg-desktop-portal-wlr.5.scd).
