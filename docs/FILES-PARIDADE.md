# Paridade funcional do Sparky Files

Acompanha a issue #90. Este documento mapeia cada ação alcançável por menu,
atalho ou clique no Sparky Files. Para cada uma, indica o equivalente na
referência, a situação e o teste que a protege.

- **Sparky Files:** `main` em `91fe62b`.
- **Referência:** COSMIC Files em `f120c0c`. Os arquivos citados no lado da
  referência são `src/key_bind.rs`, `src/menu.rs`, `src/app.rs`, `src/tab.rs`,
  `src/dialog.rs` e `src/config.rs`.
- **Revalidação:** cada branch das etapas 2 a 7 revalida esta matriz contra a
  `main` de onde parte e atualiza este arquivo.

Legenda da situação:

- **igual:** mesmo fluxo observável, mesmo que por outro atalho ou menu.
- **lacuna:** a referência oferece e o Sparky não. A coluna "Etapa" diz em
  qual etapa da issue entra.
- **divergência:** o Sparky faz diferente de propósito. A lista completa está
  na seção 4 e depende de decisão do mantenedor.
- **só no Sparky:** fluxo que a referência não tem e que deve ser preservado.

Os testes ficam em `crates/sparky-files/src/`. A tabela cita o arquivo; o nome
de cada teste está no próprio arquivo.

## 1. Gerenciador

### 1.1 Navegar e selecionar

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Mover cursor | setas, `j`/`k`, PageUp/PageDown, `Ctrl+U` | setas, PageUp/PageDown | igual | | `app/keys_table_tests.rs`, `domain/grid_tests.rs`, `app/presentation_tests.rs` |
| Primeiro e último | Home/End, `g`/`G` | Home/End | igual | | `app/keys_table_tests.rs` |
| Abrir item | Enter, `l`, seta direita na lista, duplo clique, menu "Abrir" | Enter, duplo clique, menu | igual | | `app/files_tests.rs`, `app/presentation_tests.rs` |
| Pasta superior | Backspace, `h`, `Alt+↑` | `Alt+↑` | igual | | `app/keys_table_tests.rs`, `app/files_tests.rs` |
| Pasta pessoal | `~`, `Alt+Home` | sem atalho | só no Sparky | | `app/keys_table_tests.rs` |
| Lugares da lateral | `1` a `9` | sem atalho | só no Sparky | | `app/keys_table_tests.rs`, `view/sidebar_tests.rs` |
| Histórico anterior/próximo | não existe | `Alt+←`/`Alt+→`, Backspace | lacuna | 2 | |
| Editar caminho | não existe | `Ctrl+L`, Tab completa | lacuna | 2 | |
| Segmentos do caminho | não existe; o cabeçalho mostra só o título | breadcrumb clicável com menu | lacuna | 2 | |
| Abrir em nova aba | não existe para a seleção; `Ctrl+T` abre a pasta atual | `Ctrl+Enter`, menu, clique do meio | lacuna | 2 | |
| Abrir em nova janela | não existe | `Shift+Enter`, `Ctrl+N`, menu | lacuna | 2 | |
| Abas | `Ctrl+T`, `Ctrl+W`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+PageUp`/`PageDown`, clique e × | mesmas teclas, menu Arquivo | igual | | `app/tabs_tests.rs`, `domain/tabs_tests.rs` |
| Reordenar abas | não existe | arrastar aba | lacuna | 2 | |
| Marcar item | Espaço | Ctrl+clique | divergência | | `app/files_tests.rs` |
| Seleção por intervalo | não existe; só marcar um a um | `Shift`+setas, `Shift`+clique, laço | lacuna | 2 | |
| Selecionar tudo | `Ctrl+A`, menu do fundo | `Ctrl+A`, menu Editar e fundo | igual | | `domain/context_rows_tests.rs` |
| Filtro da listagem | `/` | digitar, conforme o modo de digitação | divergência | 3 | `app/files_tests.rs`, `domain/listing_tests.rs` |
| Atualizar pasta | F5, `Ctrl+R`, aviso de mudança | F5, menu Arquivo | igual | | `app/folder_change_tests.rs` |
| Fechar | `q`, `Ctrl+Q` | `Ctrl+Q`, menu | igual | | `app/keys_table_tests.rs` |
| Escape em camadas | menu, diálogo, operação, filtro, busca, marcações | diálogo, galeria, detalhes, caminho, seleção, busca | igual | | `app/context_tests.rs`, `app/search_tests.rs` |

### 1.2 Visualizar e configurar

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Grade e lista | `Ctrl+1` grade, `Ctrl+2` lista, botões do cabeçalho | `Ctrl+1` lista, `Ctrl+2` grade, menu Ver | divergência | | `app/keys_table_tests.rs`, `app/presentation_tests.rs` |
| Ocultos | `.`, `Ctrl+H` | `Ctrl+H`, menu Ver | igual | | `domain/listing_tests.rs` |
| Ordenar | `s` troca, `S` inverte, título da coluna na lista | menu Ordenar, coluna, menu do fundo | igual | | `app/sort_tests.rs`, `domain/order_tests.rs` |
| Ordenar por tipo | `s` | não existe | só no Sparky | | `domain/order_tests.rs` |
| Pastas primeiro | sempre ligado | opção do menu Ver | lacuna | 2 | `domain/order_tests.rs` |
| Ordenação por pasta | uma ordem para tudo | lembrada por caminho | lacuna | 2 | |
| Zoom por modo | não existe | `Ctrl+=`, `Ctrl+-`, `Ctrl+0`, menu Ver | lacuna | 2 | |
| Clique simples | não existe | opção nas Configurações | lacuna | 2 | |
| Formato de hora | não segue o ambiente | segue a configuração do relógio | lacuna | 2 | |
| Página de configurações | não existe | `Ctrl+,` | lacuna | 2 | |
| Menu de aplicação (Arquivo, Editar, Ver, Ordenar) | não existe; as ações estão em atalhos e menus de contexto | barra de menus | lacuna | 2 | |

### 1.3 Buscar

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Busca recursiva na pasta | `Ctrl+F` | `Ctrl+F`, digitar | igual | | `app/search_tests.rs`, `adapters/finder_tests.rs` |
| Busca respeita ocultos | sim | sim | igual | | `adapters/finder_tests.rs` |
| Resultado antigo é descartado | sim | sim | igual | | `app/search_tests.rs`, `app/tabs_tests.rs` |
| Busca em Recentes e Lixeira | não existe | sim | lacuna | 3 | |
| Abrir local do resultado | não existe | menu "Abrir local do item" | lacuna | 3 | |
| Modos de digitação | não existe | busca, editar caminho ou selecionar por prefixo | lacuna | 3 | |
| Renomear e criar nos resultados | bloqueados | não oferecidos | igual | | `app/search_tests.rs` |

### 1.4 Inspecionar itens

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Miniaturas com cache | automático | automático | igual | | `app/previews_tests.rs`, `adapters/thumbnail_cache_tests.rs` |
| Propriedades | `Alt+Enter`, menu | `Ctrl+Espaço`, painel de detalhes | igual | | `app/context_tests.rs`, `app/details_tests.rs` |
| Galeria | não existe | Espaço, menu Ver | lacuna | 3 | |
| Detalhes de seleção múltipla | não existe | painel de detalhes | lacuna | 3 | |
| Tamanho e contagem de pastas | não existe | sim | lacuna | 3 | |
| Checksum | não existe | sob demanda | lacuna | 3 | |
| Editar permissões | não existe; só leitura | dono, grupo e outros | lacuna | 3 | |
| Trocar aplicativo padrão | pelo "Abrir com", em "usar sempre" | pelo painel de detalhes | igual | | `app/open_with_tests.rs` |

### 1.5 Executar operações

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Nova pasta | `Ctrl+Shift+N`, `a`, menu do fundo | `Ctrl+Shift+N`, menus | igual | | `app/context_tests.rs`, `operations_tests.rs` |
| Novo arquivo | `a`, menu do fundo | menus | igual | | `app/context_tests.rs`, `operations_tests.rs` |
| Renomear | F2, `r`, menu | F2, menus | igual | | `app/files_tests.rs`, `operations_tests.rs` |
| Renomear em lote | `r` com marcados | não existe | só no Sparky | | `app/batch_tests.rs`, `batch_rename_tests.rs`, `domain/batch_name_tests.rs` |
| Copiar e mover para | menu | menus | igual | | `app/menu_actions_tests.rs` |
| Lixeira | Delete, `d`, menu | Delete, menus | igual | | `app/removal_tests.rs` |
| Excluir de vez | `Shift+Delete`, menu | `Shift+Delete`, menu com Shift | igual | | `app/removal_tests.rs`, `operations_tests/delete_tests.rs` |
| Arrastar e soltar | itens, abas, lateral, lixeira; para outros apps | itens, abas, lateral | igual | | `app/drop_tests.rs`, `app/handoff_tests.rs` |
| Progresso e cancelar | barra de status, Esc | painel de operações | igual | | `app/jobs_tests.rs`, `transfer_job_tests.rs` |
| Conflitos | manter os dois, substituir, pular | os mesmos, com "aplicar aos demais" | igual | | `app/jobs_tests.rs`, `operations_tests/paste_tests.rs` |
| Desfazer | `Ctrl+Z`, aviso da lixeira | aviso da lixeira, página de histórico | igual | | `app/undo_tests.rs`, `app/removal_tests.rs` |
| Operações simultâneas | recusa a segunda | fila com várias | lacuna | 4 | `app/jobs_tests.rs` |
| Pausar e retomar | não existe | por operação e todas | lacuna | 4 | |
| Histórico de operações | não existe | página "Histórico" | lacuna | 4 | |
| Aplicar decisão aos demais | não existe | caixa no diálogo de substituir | lacuna | 4 | |
| Andamento com a janela fechada | não existe | mantém o processo | lacuna | 4 | |
| Enviar por e-mail | menu | não existe | só no Sparky | | `app/menu_actions_tests.rs` |

### 1.6 Compactar e extrair

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Compactar | menu, gera tar.gz | menu, ZIP ou tgz, senha opcional | lacuna parcial | 4 | `app/menu_actions_tests.rs` |
| Extrair aqui | `e`, menu do compactado | menu | igual | | `app/extract_tests.rs`, `adapters/extract_tests.rs` |
| Extrair em destino escolhido | não existe | menu "Extrair para" | lacuna | 4 | |
| Senha | não existe | pede senha em ZIP | lacuna | 4 | |
| Formatos de extração | tar, gz, bz2, xz, zst, lz4, zip, 7z, rar | tar, gz, bz2, xz, zip | só no Sparky (mais formatos) | | `adapters/extract_tests.rs` |

### 1.7 Clipboard e abertura

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Copiar e recortar arquivos | `Ctrl+C`/`y`, `Ctrl+X`/`x`, menu | `Ctrl+C`, `Ctrl+X`, menus | igual | | `app/clipboard_files_tests.rs`, `adapters/clipboard_tests.rs` |
| Colar | `Ctrl+V`, `p`, menu do fundo | `Ctrl+V`, menus | igual | | `app/clipboard_files_tests.rs`, `operations_tests/paste_tests.rs` |
| Imagem no clipboard Wayland | `Ctrl+C` numa imagem | não existe | só no Sparky | | `app/clipboard_tests.rs` |
| Copiar caminho como texto | não existe | `Ctrl+Shift+C`, menu com Shift | lacuna | 4 | |
| Colar conteúdo externo como arquivo | não existe | texto, imagem e vídeo | lacuna | 4 | `app/clipboard_files_tests.rs` |
| Abrir com | menu, lista com busca e "usar sempre" | menu, diálogo | igual | | `app/open_with_tests.rs` |
| Abrir no terminal | não existe | menu | lacuna | 5 | |
| Ações de entradas `.desktop` | não existe | menu | lacuna | 5 | |
| Tornar executável e abrir | não existe | diálogo de confirmação | lacuna | 5 | |
| Ações personalizadas | não existe | lista no config, por tipo de seleção | lacuna | 5 | |
| Busca na loja | não existe | não existe | lacuna (pedido da issue) | 5 | |

### 1.8 Lateral, recentes e lixeira

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Adicionar marcador | `Ctrl+D`, arrastar para a lateral | `Ctrl+D`, menus | igual | | `app/bookmark_keys_tests.rs`, `app/bookmarks_tests.rs` |
| Reordenar marcadores | arrastar | arrastar | igual | | `app/bookmarks_tests.rs`, `domain/bookmarks_tests.rs` |
| Cores dos marcadores | menu do marcador | não existe | só no Sparky | | `app/bookmark_menu_tests.rs`, `domain/context_rows_tests.rs` |
| Remover marcador | menu do marcador | menu da lateral | igual | | `app/bookmark_menu_tests.rs` |
| Renomear rótulo | não existe | menu da lateral | lacuna | 5 | `adapters/bookmark_file_tests.rs` (lê rótulos) |
| Marcador de pasta que sumiu | avisa e mantém | diálogo de erro | igual | | `app/bookmarks_tests.rs` |
| Favoritos | botão do cabeçalho, página virtual | marcadores | igual | | `app/sidebar_tests.rs`, `adapters/sidebar_source_tests.rs` |
| Recentes | página virtual | lugar da lateral | igual | | `adapters/sidebar_source_tests.rs` |
| Ocultar Recentes | não existe | opção | lacuna | 5 | |
| Remover de Recentes | não existe | menu e limpar tudo | lacuna | 5 | |
| Abrir lixeira | lateral | lateral | igual | | `app/trash_tests.rs` |
| Restaurar | menu, cabeçalho, aviso | menu Arquivo e de contexto | igual | | `app/trash_tests.rs`, `app/restore_toast_tests.rs`, `adapters/home_trash_tests.rs` |
| Restaurar todos | menu do fundo, cabeçalho | não existe | só no Sparky | | `app/trash_tests.rs` |
| Esvaziar | cabeçalho, com confirmação | menu da lateral e do ícone | igual | | `app/empty_trash_tests.rs` |
| Excluir de vez um item da lixeira | não existe; Delete e `Shift+Delete` não agem | menu | lacuna | 5 | `app/removal_tests.rs`, `app/trash_tests.rs` |
| Ordenar por data de exclusão | não existe | coluna "Excluído em" | lacuna | 5 | |
| Lixeiras de outros discos | lateral | não existe | só no Sparky | | `app/disk_trash_tests.rs`, `adapters/disk_trash_tests.rs` |

### 1.9 Volumes e rede

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Montar e abrir volume | clique na lateral, `Alt+1` a `Alt+9` | clique na lateral | igual | | `app/volumes_tests.rs` |
| Desmontar | botão da lateral, `Alt+U` | ícone da lateral | igual | | `app/volumes_tests.rs` |
| Ejetar | botão da lateral | menu do ponto de montagem | igual | | `app/volumes_eject_tests.rs`, `view/sidebar_tests.rs` |
| Celular por MTP | lateral | lateral | igual | | `app/mtp_tests.rs` |
| Navegar em rede pelo GIO | página Rede | página Rede | igual | | `adapters/gio_listing_tests.rs`, `app/sidebar_tests.rs` |
| Autenticação pendente | não bloqueia a navegação | diálogo | igual | | `app/volume_pending_tests.rs` |
| Conectar por URI | não existe | "Adicionar unidade de rede" | lacuna | 5 | |
| Credenciais e lembrar senha | não existe | diálogo e chaveiro | lacuna | 5 | |
| Favoritar compartilhamento | não existe | favorito de rede | lacuna | 5 | |
| Protocolos disponíveis | não existe | lista do backend | lacuna | 5 | |

## 2. Seletor de arquivos

O seletor é o mesmo aplicativo aberto pelo portal (`app/chooser`). Ele aceita
só navegação, seleção e nova pasta. Bloqueia menus de contexto, lixeira, abas,
favoritos, dispositivos e o desfazer.

| Ação | Como se chega no Sparky | Referência | Situação | Etapa | Teste |
|---|---|---|---|---|---|
| Abrir arquivo, vários e pasta | portal OpenFile | OpenFile, OpenFolder e variantes múltiplas | igual | | `app/chooser/chooser_tests.rs`, `domain/choice/decision_tests.rs` |
| Salvar e salvar vários | SaveFile, SaveFiles | SaveFile | igual | | `app/chooser/chooser_tests.rs`, `app/chooser/activation_tests.rs` |
| Confirmar substituição | diálogo | diálogo | igual | | `app/chooser/chooser_tests.rs` |
| Filtros MIME e glob | lista na barra | lista | igual | | `domain/choice/filter_tests.rs`, `adapters/portal/options_tests.rs` |
| Cancelar | botão, Esc | botão, Esc | igual | | `app/chooser/chooser_tests.rs`, `adapters/portal/service_tests.rs` |
| Navegação e seleção | as teclas liberadas | as teclas comuns | igual | | `app/chooser/guard_tests.rs` |
| Nova pasta | `Ctrl+Shift+N` | `Ctrl+Shift+N`, botão | igual | | `app/chooser/guard_tests.rs` |
| Ações que mexem em arquivos | bloqueadas | não oferecidas | igual | | `app/chooser/guard_tests.rs`, `app/chooser/chooser_tests.rs` |
| Opções extras (choices) | não lê nem devolve | caixas e listas, devolvidas ao solicitante | lacuna | 6 | |
| Busca | `Ctrl+F` bloqueado | `Ctrl+F` | lacuna | 6 | `app/chooser/guard_tests.rs` |
| Detalhes e galeria | bloqueados | painel e galeria | lacuna | 6 | |
| Preferências do seletor | as do gerenciador | próprias (ocultos, modo, zoom, detalhes) | lacuna | 6 | |
| Botão com mnemônico | rótulo sem sublinhado | `Alt`+letra | divergência | | `domain/choice/decision_tests.rs` |
| Mostrar itens (FileManager1) | D-Bus | D-Bus | igual | | `adapters/file_manager1_tests.rs`, `app/reveal_tests.rs` |

## 3. Desktop

O Sparky Files não tem modo desktop. Wallpaper e telas são do compositor e do
Settings. Toda a linha é lacuna da etapa 6, que primeiro define a
responsabilidade de cada componente.

| Ação da referência | Situação | Etapa |
|---|---|---|
| Ícones da pasta Desktop, volumes e lixeira na área de trabalho | lacuna | 6 |
| Visibilidade, tamanho e espaçamento dos ícones | lacuna | 6 |
| Menu do fundo com papel de parede, aparência e telas | lacuna (encaminhar ao Settings) | 6 |
| Atalhos de arquivo no desktop (copiar, recortar, colar, renomear, excluir) | lacuna | 6 |
| Pré-visualização em janela própria | lacuna | 6 |

## 4. Divergências para decisão do mantenedor

Estes atalhos e fluxos do Sparky diferem da referência. A issue pede
preservar os atuais, e os testes da seção 5 os travam. Qualquer mudança
precisa de decisão explícita.

1. `Ctrl+1` é grade e `Ctrl+2` é lista. Na referência é o contrário.
2. Espaço marca o item. Na referência abre a galeria. A galeria da etapa 3
   precisa de outra entrada.
3. Backspace sobe para a pasta superior. Na referência volta no histórico. O
   histórico da etapa 2 precisa de outra tecla para voltar, como `Alt+←`.
4. Letras sozinhas são atalhos (`j`, `k`, `d`, `y`, `p` e outras). Na
   referência, digitar busca, edita o caminho ou seleciona por prefixo. Os
   modos de digitação da etapa 3 têm que conviver com isso.
5. O filtro é por `/`. Na referência é a digitação.
6. O botão do seletor não tem mnemônico com `Alt`.

## 5. Testes de regressão desta etapa

Os quatro arquivos abaixo travam o comportamento atual que ainda não tinha
teste completo. Cada um foi conferido quebrando o código de propósito: o
teste falhou e o código foi restaurado.

| Arquivo | O que trava |
|---|---|
| `app/keys_table_tests.rs` | a tabela inteira de atalhos (letras, `Ctrl`, `Alt` e teclas nomeadas) e as teclas que continuam livres |
| `app/menu_shortcuts_tests.rs` | todo atalho exibido num menu de contexto executa a mesma ação do menu |
| `domain/context_rows_tests.rs` | ordem, rótulo, atalho e estado de cada linha de cada menu de contexto |
| `app/chooser/guard_tests.rs` | o que o seletor libera e bloqueia, comando por comando e mensagem por mensagem |

## 6. Histórico aproveitado

| Origem | Cenário | Teste |
|---|---|---|
| #14 | menus de contexto, miniaturas e ícones nítidos | `app/context_tests.rs`, `domain/context_rows_tests.rs`, `app/previews_tests.rs`, `domain/icons_tests.rs` |
| #15 | lixeira e restauração segura | `app/trash_tests.rs`, `adapters/home_trash_tests.rs` |
| #16 | imagem no clipboard Wayland | `app/clipboard_tests.rs` |
| #20 | esvaziar com confirmação | `app/empty_trash_tests.rs` |
| #35 | lateral e dispositivos | `app/volumes_tests.rs`, `view/sidebar_tests.rs`, `app/mtp_tests.rs` |
| #43 | arrastar para outros aplicativos | `app/handoff_tests.rs`, `app/drop_tests.rs` |
| #45 | seletor de arquivos | `app/chooser/chooser_tests.rs`, `domain/choice/decision_tests.rs`, `adapters/portal/options_tests.rs` |
