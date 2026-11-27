# Paridade funcional do Sparky Files

Etapa 1 da issue #90: cenários de regressão. Este documento mapeia cada ação
alcançável nos menus e atalhos nos modos gerenciador, seletor e desktop. Cada
ação registra o equivalente na referência, a situação e os testes existentes
que a protegem.

## Base

- **Snapshot da issue:** Sparky Files na `main` em `9b6245b`.
- **Main de partida desta branch:** `91fe62b`. Entre `9b6245b` e `91fe62b` não
  há mudança em `crates/sparky-files`, então a matriz vale para os dois.
- **Referência:** COSMIC Files em `f120c0c`. Fontes consultadas:
  `src/key_bind.rs`, `src/menu.rs`, `src/app.rs`, `src/tab.rs`,
  `src/dialog.rs` e `src/config.rs`.
- **Revalidação:** a matriz é revalidada contra a `main` de partida de cada
  branch.

Situação:

- **igual:** equivalência; o mesmo fluxo existe nos dois, mesmo que por outra
  tecla ou outro menu.
- **lacuna:** a referência oferece e o Sparky não.
- **sem equivalente na referência:** ação do Sparky que a referência não tem.

Testes: caminho relativo a `crates/sparky-files/src/`, no formato
`arquivo: teste`. Um traço (—) indica que não há teste existente.

## 1. Gerenciador

### 1.1 Atalhos do Sparky

Fonte: `app/keys.rs`.

| Tecla | Ação | Referência | Situação | Testes existentes |
|---|---|---|---|---|
| `↓`, `j` | descer | `↓` | igual | `app/keys_tests.rs`: `hjkl_e_setas_navegam`; `domain/grid_tests.rs`: `setas_respeitam_colunas_e_nao_atravessam_as_bordas_da_linha` |
| `↑`, `k` | subir | `↑` | igual | `app/keys_tests.rs`: `hjkl_e_setas_navegam`; `domain/listing_tests.rs`: `o_cursor_para_nas_pontas` |
| `←`, `→` | mover na grade | `←`, `→` | igual | `app/presentation_tests.rs`: `setas_movem_por_celulas_e_h_l_continuam_entrando_e_voltando` |
| `PageDown` | página abaixo | `PageDown` | igual | `app/keys_tests.rs`: `ctrl_d_marca_a_pasta_e_page_down_continua_descendo`; `domain/grid_tests.rs`: `pagina_respeita_a_capacidade_e_janelas_pequenas_preservam_uma_celula` |
| `PageUp`, `Ctrl+U` | página acima | `PageUp` | igual | `domain/grid_tests.rs`: `pagina_respeita_a_capacidade_e_janelas_pequenas_preservam_uma_celula` |
| `Home`, `g` | primeiro item | `Home` | igual | `app/keys_tests.rs`: `g_vai_ao_comeco_e_g_maiusculo_ao_fim` |
| `End`, `G` | último item | `End` | igual | `app/keys_tests.rs`: `g_vai_ao_comeco_e_g_maiusculo_ao_fim` |
| `Enter`, `l` | abrir | `Enter` | igual | `app/keys_tests.rs`: `hjkl_e_setas_navegam`; `app/files_tests.rs`: `abrir_um_arquivo_chama_o_aplicativo_padrao` |
| `Backspace`, `h`, `Alt+↑` | pasta superior | `Alt+↑` (na referência, `Backspace` volta no histórico) | igual | `app/keys_tests.rs`: `alt_com_seta_para_cima_sobe_e_alt_home_vai_para_casa`; `app/files_tests.rs`: `entrar_e_voltar_deixa_a_pasta_de_onde_veio_selecionada` |
| `~`, `Alt+Home` | pasta pessoal | — | sem equivalente na referência | `app/keys_tests.rs`: `alt_com_seta_para_cima_sobe_e_alt_home_vai_para_casa` |
| `1` a `9` | lugar da lateral | — | sem equivalente na referência | `app/keys_tests.rs`: `numeros_levam_aos_lugares_da_lateral`; `view/sidebar_tests.rs`: `ordem_visual_preserva_indices_dos_atalhos` |
| `Alt+1` a `Alt+9` | abrir volume | — | sem equivalente na referência | `app/keys_tests.rs`: `visualizacoes_e_dispositivos_tem_atalhos_separados_dos_lugares` |
| `Alt+U` | desmontar o volume aberto | — | sem equivalente na referência | `app/volumes_tests.rs`: `atalho_desmonta_apenas_o_volume_em_que_esta_navegando` |
| `.`, `Ctrl+H` | mostrar ocultos | `Ctrl+H` | igual | `domain/listing_tests.rs`: `esconde_os_ocultos_e_mostra_quando_pedido_mantendo_a_selecao` |
| `/` | filtrar a listagem | — | sem equivalente na referência | `app/files_tests.rs`: `o_filtro_reduz_a_lista_e_esc_limpa`; `app/keys_tests.rs`: `no_filtro_so_as_setas_movem`; `domain/listing_tests.rs`: `o_filtro_ignora_caixa_e_volta_o_cursor_ao_comeco` |
| `Espaço` | marcar item | `Ctrl`+clique marca; `Espaço` abre a galeria | igual | `app/files_tests.rs`: `marcar_com_espaco_e_copiar_leva_todos_os_marcados`; `domain/listing_tests.rs`: `marcar_de_novo_desmarca_e_as_marcas_viram_a_escolha` |
| `Ctrl+C`, `y` | copiar | `Ctrl+C` | igual | `app/keys_tests.rs`: `atalhos_de_arquivo_por_letra_e_por_ctrl`; `app/clipboard_files_tests.rs`: `copiar_e_recortar_publicam_a_lista_de_arquivos` |
| `Ctrl+X`, `x` | recortar | `Ctrl+X` | igual | `app/keys_tests.rs`: `atalhos_de_arquivo_por_letra_e_por_ctrl`; `app/clipboard_files_tests.rs`: `copiar_e_recortar_publicam_a_lista_de_arquivos` |
| `Ctrl+V`, `p` | colar | `Ctrl+V` | igual | `app/keys_tests.rs`: `atalhos_de_arquivo_por_letra_e_por_ctrl`; `app/files_tests.rs`: `copiar_e_colar_na_mesma_pasta_duplica` |
| `F2`, `r` | renomear (em lote com marcados) | `F2` | igual | `app/files_tests.rs`: `renomear_comeca_com_o_nome_atual`; `app/batch_tests.rs`: `r_com_varios_marcados_renomeia_em_lote_na_ordem_da_lista` |
| `a` | criar pasta ou arquivo pela barra | — | sem equivalente na referência | `app/keys_tests.rs`: `atalhos_de_arquivo_por_letra_e_por_ctrl`; `operations_tests.rs`: `criar_pasta_ou_arquivo_conforme_a_barra` |
| `Ctrl+Shift+N` | nova pasta | `Ctrl+Shift+N` | igual | `app/keys_tests.rs`: `atalhos_exibidos_nos_menus_de_contexto_existem`; `app/files_tests.rs`: `criar_pasta_pelo_prompt_seleciona_a_nova` |
| `Ctrl+A` | selecionar tudo | `Ctrl+A` | igual | `app/keys_tests.rs`: `atalhos_exibidos_nos_menus_de_contexto_existem` |
| `Alt+Enter` | propriedades | `Ctrl+Espaço` (detalhes) | igual | `app/keys_tests.rs`: `atalhos_exibidos_nos_menus_de_contexto_existem`; `app/context_tests.rs`: `propriedades_do_item_e_da_area_vazia_mostram_o_alvo_correto` |
| `Delete`, `d` | mover para a lixeira | `Delete` | igual | `app/removal_tests.rs`: `delete_manda_para_a_lixeira_sem_perguntar_e_mostra_o_aviso`; `app/files_tests.rs`: `d_manda_para_a_lixeira_sem_confirmacao` |
| `Shift+Delete` | excluir de vez | `Shift+Delete` | igual | `app/removal_tests.rs`: `shift_delete_abre_a_confirmacao_e_cancelar_mantem_o_item`, `confirmar_exclui_de_vez_sem_passar_pela_lixeira` |
| `s` | próximo critério de ordenação | menu Ordenar | igual | `app/sort_tests.rs`: `s_troca_o_criterio_e_s_maiusculo_inverte`; `domain/order_tests.rs`: `trocar_de_criterio_percorre_todos_e_inverter_alterna` |
| `S` | inverter a ordem | menu Ordenar | igual | `app/sort_tests.rs`: `s_troca_o_criterio_e_s_maiusculo_inverte` |
| `e` | extrair aqui | — | sem equivalente na referência | `app/extract_tests.rs`: `e_extrai_aqui_seleciona_o_resultado_e_ctrl_z_desfaz`, `e_sobre_arquivo_que_nao_e_compactado_nao_faz_nada` |
| `Ctrl+F` | busca recursiva | `Ctrl+F` | igual | `app/search_tests.rs`: `ctrl_f_busca_nas_subpastas_e_mostra_o_caminho_relativo` |
| `Ctrl+T` | nova aba | `Ctrl+T` | igual | `app/tabs_tests.rs`: `ctrl_t_abre_outra_aba_na_mesma_pasta_e_cada_uma_navega_sozinha` |
| `Ctrl+W` | fechar aba | `Ctrl+W` | igual | `app/tabs_tests.rs`: `ctrl_w_fecha_a_aba_atual_e_a_ultima_continua_aberta` |
| `Ctrl+Tab`, `Ctrl+PageDown` | próxima aba | `Ctrl+Tab` | igual | `app/keys_tests.rs`: `atalhos_de_abas_usam_ctrl` |
| `Ctrl+Shift+Tab`, `Ctrl+PageUp` | aba anterior | `Ctrl+Shift+Tab` | igual | `app/keys_tests.rs`: `atalhos_de_abas_usam_ctrl` |
| `Ctrl+Z` | desfazer | página Histórico e aviso | igual | `app/keys_tests.rs`: `ctrl_z_desfaz`; `app/undo_tests.rs`: `desfazer_renomear_volta_ao_nome_antigo`, `sem_historico_avisa_que_nao_ha_o_que_desfazer` |
| `F5`, `Ctrl+R` | reler a pasta | `F5` | igual | — |
| `q`, `Ctrl+Q` | fechar a janela | `Ctrl+Q` | igual | — |
| `Ctrl+1` | grade | `Ctrl+2` | igual | `app/keys_tests.rs`: `visualizacoes_e_dispositivos_tem_atalhos_separados_dos_lugares` |
| `Ctrl+2` | lista | `Ctrl+1` | igual | `app/keys_tests.rs`: `visualizacoes_e_dispositivos_tem_atalhos_separados_dos_lugares` |
| `Ctrl+D` | adicionar marcador | `Ctrl+D` | igual | `app/bookmark_keys_tests.rs`: `ctrl_d_marca_a_pasta_atual_e_grava_no_arquivo_do_sparky`, `ctrl_d_numa_pasta_ja_marcada_nao_duplica`, `ctrl_d_com_uma_pasta_selecionada_marca_a_pasta_e_nao_a_aberta`, `ctrl_d_com_so_arquivos_marcados_avisa_e_nao_marca_nada` |
| `Esc` | fechar arraste, diálogo, menu ou operação; limpar filtro, busca ou marcas | `Esc` | igual | `app/context_tests.rs`: `clique_na_area_vazia_e_escape_nao_mudam_a_selecao`; `app/search_tests.rs`: `esc_volta_para_a_pasta_e_reler_nao_desfaz_os_resultados`; `app/jobs_tests.rs`: `esc_pede_o_cancelamento_da_operacao`; `app/drop_tests.rs`: `soltar_fora_de_um_alvo_ou_cancelar_com_esc_nao_faz_nada` |
| `Enter`, `s`, `y` / `n` | confirmar / recusar | botões do diálogo | igual | `app/keys_tests.rs`: `a_confirmacao_aceita_enter_s_e_recusa_n` |
| `Enter`, `m` / `s` / `p` | conflito: manter os dois / substituir / pular | diálogo Substituir | igual | `app/jobs_tests.rs`: `manter_os_dois_cria_uma_copia_com_outro_nome`, `substituir_troca_o_item_e_guarda_o_antigo_na_lixeira`, `pular_mantem_o_item_do_destino` |
| teclas sem função | ignoradas | — | igual | `app/keys_tests.rs`: `teclas_sem_funcao_sao_ignoradas`; `app/context_tests.rs`: `teclas_nao_agem_por_tras_de_menu_ou_dialogo_aberto` |

### 1.2 Menus de contexto do Sparky

Fonte: `domain/context.rs`. Regras gerais:

- `app/menu_actions_tests.rs`: `acao_desabilitada_ou_de_outro_menu_nao_e_executada`.
- `app/context_tests.rs`: `atalhos_do_teclado_executam_as_acoes_dos_menus`.
- `app/shell_tests.rs`: `clique_direito_abre_o_menu_como_popup_da_janela`.

| Menu | Item | Referência | Situação | Testes existentes |
|---|---|---|---|---|
| arquivo, pasta, compactado | Abrir | Abrir | igual | `domain/context_tests.rs`: `arquivo_e_pasta_expoem_operacoes_reais_e_propriedades` |
| arquivo, pasta, compactado | Abrir com | Abrir com | igual | `app/menu_actions_tests.rs`: `abrir_com_lista_aplicativos_e_abre_somente_uma_escolha_valida` |
| arquivo, pasta, compactado | Recortar | Recortar | igual | `domain/context_tests.rs`: `arquivo_e_pasta_expoem_operacoes_reais_e_propriedades` |
| arquivo, pasta, compactado | Copiar | Copiar | igual | `domain/context_tests.rs`: `arquivo_e_pasta_expoem_operacoes_reais_e_propriedades` |
| arquivo, pasta, compactado | Mover para | Mover para | igual | `app/menu_actions_tests.rs`: `copiar_e_mover_para_outra_pasta_nao_alteram_clipboard`, `destino_invalido_nao_apaga_a_origem` |
| arquivo, pasta, compactado | Copiar para | Copiar para | igual | `app/menu_actions_tests.rs`: `copiar_e_mover_para_outra_pasta_nao_alteram_clipboard` |
| arquivo, pasta, compactado | Renomear | Renomear | igual | `domain/context_tests.rs`: `arquivo_e_pasta_expoem_operacoes_reais_e_propriedades` |
| arquivo, pasta, compactado | Compactar (tar.gz) | Compactar (ZIP ou tgz) | igual | `app/menu_actions_tests.rs`: `comprimir_cria_arquivo_sem_bloquear_o_ciclo_de_eventos` |
| compactado | Extrair aqui | Extrair aqui | igual | `app/extract_tests.rs`: `menu_de_um_compactado_oferece_extrair_aqui`; `domain/context_tests.rs`: `compactado_e_um_arquivo_com_extrair_aqui` |
| arquivo, compactado (pasta desabilitado) | Enviar por e-mail | — | sem equivalente na referência | `app/menu_actions_tests.rs`: `enviar_arquivo_por_email_e_impedir_pasta` |
| arquivo, pasta, compactado | Mover para a lixeira | Mover para a lixeira | igual | `domain/context_tests.rs`: `arquivo_e_pasta_expoem_operacoes_reais_e_propriedades` |
| arquivo, pasta, compactado | Excluir de vez | Excluir de vez (com `Shift`) | igual | `app/removal_tests.rs`: `menu_de_contexto_oferece_excluir_de_vez` |
| arquivo, pasta, compactado | Propriedades | Mostrar detalhes | igual | `app/context_tests.rs`: `propriedades_do_item_e_da_area_vazia_mostram_o_alvo_correto` |
| área vazia | Nova pasta | Nova pasta | igual | `app/context_tests.rs`: `novas_pasta_e_documento_preservam_o_tipo_ao_editar_o_nome` |
| área vazia | Novo arquivo | Novo arquivo | igual | `app/context_tests.rs`: `novas_pasta_e_documento_preservam_o_tipo_ao_editar_o_nome` |
| área vazia | Abrir com | — | sem equivalente na referência | — |
| área vazia | Colar (desabilitado sem clipboard) | Colar | igual | `domain/context_tests.rs`: `area_vazia_oferece_criacao_e_so_habilita_colar_com_clipboard` |
| área vazia | Selecionar tudo (desabilitado sem itens) | Selecionar tudo | igual | `domain/context_tests.rs`: `area_vazia_oferece_criacao_e_so_habilita_colar_com_clipboard` |
| área vazia | Propriedades | — | sem equivalente na referência | `app/context_tests.rs`: `propriedades_do_item_e_da_area_vazia_mostram_o_alvo_correto` |
| item da lixeira | Restaurar | Restaurar da lixeira | igual | `app/trash_tests.rs`: `restaura_somente_o_item_escolhido_pelo_menu`; `domain/context_tests.rs`: `lixeira_oferece_apenas_restauracao_e_propriedades` |
| item da lixeira | Propriedades | Mostrar detalhes | igual | `domain/context_tests.rs`: `lixeira_oferece_apenas_restauracao_e_propriedades` |
| área vazia da lixeira | Restaurar todos | — | sem equivalente na referência | `app/trash_tests.rs`: `restaurar_todos_preserva_conflitos_e_restaura_os_demais`, `restaurar_todos_inclui_itens_ocultos_pela_visualizacao` |
| marcador | cores da paleta | — | sem equivalente na referência | `app/bookmark_menu_tests.rs`: `escolher_uma_cor_pinta_o_marcador_na_hora_e_grava` |
| marcador | Sem cor | — | sem equivalente na referência | `app/bookmark_menu_tests.rs`: `sem_cor_volta_ao_padrao_e_a_cor_sobrevive_a_releitura` |
| marcador | Remover marcador | Remover da lateral | igual | `app/bookmark_menu_tests.rs`: `remover_tira_da_lateral_e_do_arquivo_sem_tocar_na_pasta`, `a_acao_vale_para_o_marcador_clicado_mesmo_se_o_arquivo_mudou_por_fora` |

### 1.3 Mouse, cabeçalho, lateral, abas e avisos do Sparky

| Entrada | Ação | Referência | Situação | Testes existentes |
|---|---|---|---|---|
| clique no item | selecionar | clique | igual | `app/presentation_tests.rs`: `clique_usa_indice_absoluto_depois_da_rolagem` |
| duplo clique no item | abrir | duplo clique | igual | `app/presentation_tests.rs`: `ativacao_invalida_nao_abre_a_pasta_selecionada` |
| clique direito no item | menu do item | clique direito | igual | `app/context_tests.rs`: `clique_direito_seleciona_o_item_e_abre_o_menu_no_ponteiro`, `clique_direito_preserva_multisselecao_quando_item_esta_marcado` |
| clique direito na área vazia | menu da área vazia | clique direito | igual | `app/context_tests.rs`: `clique_na_area_vazia_e_escape_nao_mudam_a_selecao` |
| rolar | rolar a listagem | rolar | igual | `app/presentation_tests.rs`: `atualizacao_periodica_nao_desfaz_rolagem_manual`; `app/details_tests.rs`: `rolar_preenche_as_linhas_que_aparecem` |
| botões Grade e Lista | trocar o modo | menu Ver | igual | `app/presentation_tests.rs`: `alternar_grade_e_lista_preserva_filtro_selecao_e_marcacoes` |
| título da coluna na lista | ordenar; de novo inverte | título da coluna | igual | `app/sort_tests.rs`: `clicar_no_titulo_escolhe_o_criterio_e_de_novo_inverte` |
| cabeçalho da lixeira | Restaurar selecionado | menu Arquivo | igual | `app/trash_tests.rs`: `restaura_somente_o_item_escolhido_pelo_menu` |
| cabeçalho da lixeira | Restaurar todos | — | sem equivalente na referência | `app/trash_tests.rs`: `restaurar_todos_recupera_pasta_antes_do_arquivo_filho`, `restaurar_todos_encerra_quando_o_pai_nao_pode_ser_recuperado` |
| cabeçalho da lixeira | Esvaziar lixeira | Esvaziar lixeira | igual | `app/empty_trash_tests.rs`: `esvaziar_exige_confirmacao_e_inclui_itens_ocultos` |
| cabeçalho | Favoritar | Adicionar à lateral | igual | `adapters/sidebar_source_tests.rs`: `favoritos_persistem_e_remocao_atualiza_a_lista` |
| página Favoritos | Remover favorito | Remover da lateral | igual | `adapters/sidebar_source_tests.rs`: `favoritos_persistem_e_remocao_atualiza_a_lista` |
| lateral | lugar | lugar | igual | `view/sidebar_tests.rs`: `ordem_visual_preserva_indices_dos_atalhos` |
| lateral | Recentes | Recentes | igual | `adapters/sidebar_source_tests.rs`: `recentes_ignoram_arquivo_removido_e_ordenam_pela_visita` |
| lateral | Favoritos | favoritos da lateral | igual | `app/sidebar_tests.rs`: `favoritos_abrem_arquivo_e_navegacao_local_limpa_pagina_virtual`, `mover_favorito_mantem_referencia_no_destino` |
| lateral | Rede | Rede | igual | `app/sidebar_tests.rs`: `consulta_de_rede_roda_fora_da_thread_da_interface`, `resposta_atrasada_nao_reabre_pagina_de_rede`; `adapters/sidebar_source_tests.rs`: `rede_lista_arquivos_por_uri_gio`, `atalho_de_rede_abre_a_uri_de_destino` |
| lateral | Lixeira | Lixeira | igual | `app/trash_tests.rs`: `abre_lixeira_vazia_e_volta_para_a_pasta_pessoal` |
| lateral | lixeira de outro disco | — | sem equivalente na referência | `app/disk_trash_tests.rs`: `lixeira_de_outro_disco_abre_e_restaura_no_lugar_original`, `lixeira_pessoal_continua_separada_das_outras` |
| lateral | marcador | favorito | igual | `app/bookmarks_tests.rs`: `clicar_num_marcador_abre_a_pasta`, `marcador_de_pasta_que_sumiu_avisa_e_fica_onde_esta` |
| lateral | arrastar marcador sobre outro | — | sem equivalente na referência | `app/bookmarks_tests.rs`: `arrastar_um_marcador_sobre_outro_reordena_e_grava`, `esc_ou_soltar_fora_de_um_marcador_nao_reordena` |
| lateral | abrir volume | clique no volume | igual | `app/volumes_tests.rs`: `volume_montado_abre_sem_disparar_comando_de_montagem`, `montagem_assincrona_abre_o_ponto_retornado_e_serializa_pedidos`, `recusa_de_montagem_preserva_localizacao_e_mostra_erro` |
| lateral | desmontar volume | ícone de ejeção | igual | `app/volumes_tests.rs`: `remover_ou_desmontar_volume_atual_volta_para_pasta_pessoal` |
| lateral | ejetar volume | Ejetar | igual | `app/volumes_eject_tests.rs`: `ejecao_e_aceita_so_para_volume_removivel_e_serializada`; `view/sidebar_tests.rs`: `ejetar_so_e_oferecido_para_unidade_removivel_de_bloco` |
| lateral | celular por MTP | celular por MTP | igual | `app/mtp_tests.rs`: `clicar_no_android_abre_o_ponto_local_retornado_pelo_backend`, `retirar_o_android_aberto_volta_para_a_pasta_pessoal` |
| aba | trocar de aba | clique na aba | igual | `app/tabs_tests.rs`: `voltar_para_uma_aba_rele_a_pasta_e_mantem_a_selecao` |
| × da aba | fechar aba | × da aba | igual | `app/tabs_tests.rs`: `clicar_no_x_de_outra_aba_fecha_so_ela` |
| aviso da lixeira | Desfazer | Desfazer | igual | `app/removal_tests.rs`: `desfazer_pelo_aviso_restaura_o_item_e_fecha_o_aviso`, `desfazer_pelo_aviso_nao_desfaz_uma_acao_mais_nova` |
| aviso da restauração | Abrir pasta | — | sem equivalente na referência | `app/restore_toast_tests.rs`: `abrir_pasta_pelo_aviso_vai_ate_o_item_restaurado` |
| × do aviso | fechar aviso | fechar aviso | igual | `app/removal_tests.rs`: `o_aviso_fecha_no_x_e_sozinho_depois_de_alguns_segundos` |
| arrastar item sobre pasta | mover | arrastar | igual | `app/drop_tests.rs`: `soltar_um_arquivo_sobre_uma_pasta_move_e_pode_desfazer`, `uma_pasta_nao_pode_ser_solta_sobre_ela_mesma_nem_sobre_arquivo` |
| arrastar com `Ctrl` | copiar | arrastar com `Ctrl` | igual | `app/drop_tests.rs`: `com_ctrl_pressionado_soltar_copia` |
| arrastar para lugar ou aba | mover para a pasta dele | arrastar para lateral ou aba | igual | `app/drop_tests.rs`: `soltar_num_lugar_da_lateral_ou_numa_aba_leva_para_a_pasta_dele` |
| arrastar para a lixeira | mover para a lixeira | arrastar para a lixeira | igual | `app/drop_tests.rs`: `soltar_na_lixeira_manda_direto_e_mostra_o_aviso` |
| arrastar pasta para marcadores | adicionar marcador | arrastar para a lateral | igual | `app/bookmarks_tests.rs`: `arrastar_uma_pasta_para_os_marcadores_adiciona_sem_mover`, `arrastar_so_arquivos_para_os_marcadores_avisa_e_nao_adiciona` |
| arrastar para fora da janela | entregar a outro aplicativo | arrastar para fora | igual | `app/handoff_tests.rs`: `sair_da_janela_arrastando_entrega_ao_compositor_uma_vez`, `arrastar_um_item_marcado_leva_todos_os_marcados_para_fora`, `recusa_do_compositor_aparece_na_barra_de_status` |
| janela Abrir com | buscar, setas, clique, duplo clique, "usar sempre" | diálogo Abrir com | igual | `app/open_with_tests.rs`: `a_janela_abre_com_o_recomendado_selecionado_e_os_outros_disponiveis`, `buscar_filtra_e_seleciona_o_primeiro_e_enter_abre_uma_vez`, `setas_mudam_a_selecao_e_usar_sempre_vai_junto`, `clicar_seleciona_e_clique_duplo_abre`, `busca_sem_resultado_nao_abre_nada` |
| barra de status | prévia da renomeação em lote | — | sem equivalente na referência | `app/batch_tests.rs`: `a_previa_mostra_os_primeiros_nomes_e_quantos_faltam` |
| barra de status | andamento da operação | painel de operações | igual | `app/jobs_tests.rs`: `barra_de_status_mostra_o_andamento` |

### 1.4 Ações da referência sem equivalente no Sparky

| Ação da referência | Como se chega na referência | Situação |
|---|---|---|
| Voltar no histórico | `Alt+←`, `Backspace` | lacuna |
| Avançar no histórico | `Alt+→` | lacuna |
| Editar o caminho | `Ctrl+L` | lacuna |
| Completar o caminho | `Tab` no caminho | lacuna |
| Navegar pelos segmentos do caminho | clique no segmento | lacuna |
| Segmento: abrir em nova aba | menu do segmento | lacuna |
| Segmento: abrir em nova janela | menu do segmento | lacuna |
| Segmento: mostrar detalhes | menu do segmento | lacuna |
| Segmento: adicionar à lateral | menu do segmento | lacuna |
| Abrir seleção em nova aba | `Ctrl+Enter`, menu, clique do meio | lacuna |
| Abrir seleção em nova janela | `Shift+Enter`, menu | lacuna |
| Nova janela | `Ctrl+N`, menu Arquivo | lacuna |
| Estender seleção | `Shift`+setas, `Shift+PageUp`/`PageDown`, `Shift+Home`/`End` | lacuna |
| Selecionar com o mouse | `Ctrl`+clique, `Shift`+clique, laço | lacuna |
| Reordenar abas | arrastar aba | lacuna |
| Barra de menus | menus Arquivo, Editar, Ver e Ordenar | lacuna |
| Ordenar pelo menu da área vazia | Ordenar por nome, modificação e tamanho | lacuna |
| Zoom maior | `Ctrl+=`, `Ctrl++`, menu Ver | lacuna |
| Zoom menor | `Ctrl+-`, menu Ver | lacuna |
| Tamanho padrão | `Ctrl+0`, menu Ver | lacuna |
| Pastas primeiro | menu Ver | lacuna |
| Mostrar detalhes (painel) | `Ctrl+Espaço`, menu Ver | lacuna |
| Galeria | `Espaço`, menu Ver | lacuna |
| Galeria: anterior e próxima | botões e setas | lacuna |
| Configurações | `Ctrl+,`, menu Ver | lacuna |
| Configurações: tema | Configurações | lacuna |
| Configurações: modo de digitação | Configurações | lacuna |
| Configurações: clique simples | Configurações | lacuna |
| Configurações: mostrar Recentes | Configurações | lacuna |
| Sobre | menu Ver | lacuna |
| Histórico de edições | menu Editar | lacuna |
| Copiar caminho | `Ctrl+Shift+C`, menu com `Shift` | lacuna |
| Colar conteúdo externo como arquivo | `Ctrl+V` com texto, imagem ou vídeo | lacuna |
| Extrair para | menu do compactado | lacuna |
| Compactar em ZIP com senha | diálogo Compactar | lacuna |
| Senha ao extrair | diálogo de senha | lacuna |
| Abrir no terminal | menu da pasta e da área vazia | lacuna |
| Abrir local do item | menu em Recentes e na busca | lacuna |
| Ações de entrada `.desktop` | menu do `.desktop` | lacuna |
| Ações personalizadas | menu, configuradas no config | lacuna |
| Confirmar ação personalizada | diálogo | lacuna |
| Tornar executável e abrir | diálogo | lacuna |
| Remover de Recentes | menu em Recentes | lacuna |
| Limpar Recentes | menu da lateral, botão na página vazia | lacuna |
| Renomear rótulo do favorito | menu da lateral | lacuna |
| Lateral: abrir em nova aba | menu da lateral, clique do meio | lacuna |
| Lateral: abrir em nova janela | menu da lateral | lacuna |
| Lateral: mostrar detalhes | menu da lateral | lacuna |
| Lateral: ações personalizadas | menu da lateral | lacuna |
| Lateral: esvaziar lixeira | menu da lateral | lacuna |
| Excluir de vez um item da lixeira | menu na lixeira | lacuna |
| Ordenar a lixeira pela data de exclusão | menu Ordenar, coluna | lacuna |
| Detalhes: trocar o aplicativo padrão | painel de detalhes | lacuna |
| Detalhes: editar permissões | painel de detalhes | lacuna |
| Detalhes: calcular e copiar checksum | painel de detalhes | lacuna |
| Pausar operação | painel de operações | lacuna |
| Pausar todas | painel de operações | lacuna |
| Cancelar uma operação entre várias | painel de operações | lacuna |
| Cancelar todas | painel de operações | lacuna |
| Dispensar operação concluída | painel de operações | lacuna |
| Aplicar a decisão de conflito aos demais | diálogo Substituir | lacuna |
| Adicionar unidade de rede por URI | botão na página Rede | lacuna |
| Autenticar na rede | diálogo de credenciais | lacuna |
| Buscar em Recentes e na Lixeira | busca nesses locais | lacuna |

## 2. Seletor de arquivos

Fontes: `app/chooser`, `domain/choice` e `adapters/portal`. O seletor
libera só as teclas de `app/chooser/guard.rs` e bloqueia os menus de
contexto, a lixeira, as abas, os favoritos, os dispositivos e o desfazer.

### 2.1 Ações do Sparky

| Entrada | Ação | Referência | Situação | Testes existentes |
|---|---|---|---|---|
| portal OpenFile | abrir um ou vários arquivos | OpenFile, OpenMultipleFiles | igual | `app/chooser/chooser_tests.rs`: `ativar_um_arquivo_devolve_o_caminho_sem_abrir_em_outro_aplicativo`; `domain/choice/decision_tests.rs`: `abrir_arquivo_devolve_o_escolhido_e_respeita_a_escolha_unica` |
| portal OpenFile com pasta | escolher pasta | OpenFolder | igual | `app/chooser/chooser_tests.rs`: `escolher_pasta_lista_so_pastas_e_sem_clique_devolve_a_atual`; `domain/choice/decision_tests.rs`: `escolher_pasta_devolve_a_selecionada_ou_a_atual` |
| portal SaveFile | salvar | SaveFile | igual | `app/chooser/chooser_tests.rs`: `salvar_usa_o_nome_e_confirma_antes_de_substituir`; `domain/choice/decision_tests.rs`: `salvar_usa_o_nome_digitado_e_pede_confirmacao_se_ja_existe` |
| portal SaveFiles | salvar vários | — | sem equivalente na referência | `app/chooser/activation_tests.rs`: `salvar_varios_sem_escolher_pasta_salva_na_pasta_aberta`, `salvar_varios_numa_pasta_clicada_salva_dentro_dela`; `domain/choice/decision_tests.rs`: `salvar_varios_coloca_os_nomes_na_pasta_escolhida` |
| campo de nome | digitar o nome | campo de nome | igual | `adapters/portal/options_tests.rs`: `salvar_usa_o_arquivo_atual_ou_o_nome_sugerido` |
| duplo clique ao salvar | copiar o nome do arquivo | — | sem equivalente na referência | `app/chooser/chooser_tests.rs`: `ao_salvar_clicar_duas_vezes_num_arquivo_copia_o_nome` |
| lista de filtros | trocar o filtro | lista de filtros | igual | `app/chooser/chooser_tests.rs`: `o_filtro_esconde_arquivos_e_trocar_de_filtro_mostra_de_novo`; `domain/choice/filter_tests.rs`: `padroes_com_asterisco_interrogacao_e_classes_ignoram_maiusculas`, `tipo_mime_aceita_exato_e_familia`; `adapters/portal/options_tests.rs`: `o_resultado_traz_uris_e_o_filtro_usado` |
| Substituir / Continuar editando | confirmar ou voltar | diálogo Substituir | igual | `app/chooser/chooser_tests.rs`: `salvar_usa_o_nome_e_confirma_antes_de_substituir` |
| botão de aceitar | abrir ou salvar | botão de aceitar | igual | `domain/choice/decision_tests.rs`: `o_rotulo_do_botao_perde_o_atalho_sublinhado` |
| Cancelar, `Esc` | cancelar | Cancelar, `Esc` | igual | `app/chooser/chooser_tests.rs`: `cancelar_ou_esc_nao_devolvem_nada`; `adapters/portal/service_tests.rs`: `cancelar_ou_falhar_responde_sem_resultados` |
| `→` na lista | arquivo devolve; pasta entra | `Enter` | igual | `app/chooser/activation_tests.rs`: `na_lista_a_seta_direita_num_arquivo_devolve_o_caminho_sem_abrir_outro_aplicativo`, `na_lista_a_seta_direita_numa_pasta_continua_entrando_nela` |
| setas, `j`/`k`, `PageUp`/`PageDown`, `Home`/`End`, `g`/`G` | navegar | as mesmas teclas | igual | — |
| `Backspace`, `h`, `Alt+↑`, `~`, `Alt+Home` | pasta superior e pessoal | `Alt+↑` | igual | — |
| `1` a `9`, `Alt+1` a `Alt+9` | lugares e volumes | — | sem equivalente na referência | — |
| `.`, `Ctrl+H` | ocultos | `Ctrl+H` | igual | — |
| `/` | filtrar | — | sem equivalente na referência | — |
| `Espaço`, `Ctrl+A` | marcar, selecionar tudo | `Ctrl+A` | igual | — |
| `s`, `S` | ordenar | menu Ordenar | igual | — |
| `F5`, `Ctrl+R` | reler | `F5` | igual | — |
| `Ctrl+1`, `Ctrl+2` | grade, lista | `Ctrl+2`, `Ctrl+1` | igual | — |
| `Ctrl+Shift+N` | nova pasta | `Ctrl+Shift+N`, botão | igual | — |
| menus, `Delete`, `Ctrl+X` e marcadores | bloqueados | não oferecidos | igual | `app/chooser/chooser_tests.rs`: `na_escolha_menus_e_teclas_que_mexem_em_arquivos_nao_funcionam` |
| resposta ao portal | devolver URIs e filtro | resposta ao portal | igual | `adapters/portal/service_tests.rs`: `escolha_feita_responde_sucesso_com_as_uris`; `adapters/portal/dialog_tests.rs`: `a_janela_que_escreve_o_resultado_devolve_a_escolha`, `janela_fechada_sem_resposta_cancela_e_resposta_quebrada_falha`, `fechar_o_pedido_encerra_a_janela_aberta` |
| opções do pedido | rótulo, múltiplos, pasta, filtro atual | opções do pedido | igual | `adapters/portal/options_tests.rs`: `abrir_arquivo_le_rotulo_multiplos_pasta_e_filtro_escolhido`, `filtro_atual_sem_lista_vira_o_unico_filtro_e_opcoes_ausentes_tem_padrao`, `salvar_varios_guarda_so_os_nomes`; `domain/choice/wire_tests.rs`: `o_pedido_vai_e_volta_igual_para_os_tres_propositos`, `caminho_que_nao_e_utf8_chega_intacto` |
| D-Bus FileManager1 | mostrar itens e pastas | FileManager1 | igual | `adapters/file_manager1_tests.rs`: `mostrar_itens_abre_a_pasta_com_selecao_e_repassa_o_token`, `pastas_e_propriedades_sem_token_abrem_janelas_proprias`; `app/reveal_tests.rs`: `revelar_abre_a_pasta_e_marca_os_itens_pedidos` |

### 2.2 Ações da referência sem equivalente no seletor do Sparky

| Ação da referência | Como se chega na referência | Situação |
|---|---|---|
| Opções extras (caixas e listas) devolvidas ao solicitante | controles do diálogo | lacuna |
| Botão de aceitar com mnemônico | `Alt`+letra sublinhada | lacuna |
| Editar o caminho | `Ctrl+L` | lacuna |
| Voltar e avançar no histórico | `Alt+←`, `Alt+→`, `Backspace` | lacuna |
| Buscar | `Ctrl+F`, botão de busca | lacuna |
| Modos de digitação | digitar | lacuna |
| Galeria | `Espaço`, menu Mais | lacuna |
| Mostrar detalhes | `Ctrl+Espaço`, menu Mais | lacuna |
| Zoom maior, menor e padrão | `Ctrl+=`, `Ctrl+-`, `Ctrl+0`, menu Mais | lacuna |
| Pastas primeiro | menu Mais | lacuna |
| Menu de visualização | botão Ver | lacuna |
| Menu de ordenação | botão Ordenar | lacuna |
| Menu de contexto: abrir, abrir local do item, mostrar detalhes | clique direito | lacuna |
| Menu de contexto da área vazia: nova pasta, selecionar tudo, ordenar | clique direito | lacuna |
| Preferências próprias do seletor | configuração do diálogo | lacuna |

## 3. Desktop

O Sparky Files não tem modo desktop. Todas as ações de desktop da referência
são lacunas.

| Ação da referência | Como se chega na referência | Situação |
|---|---|---|
| Mostrar o conteúdo da pasta Desktop | opções de visualização do desktop | lacuna |
| Mostrar volumes montados | opções de visualização do desktop | lacuna |
| Mostrar o ícone da lixeira | opções de visualização do desktop | lacuna |
| Tamanho dos ícones | opções de visualização do desktop | lacuna |
| Espaçamento da grade | opções de visualização do desktop | lacuna |
| Abrir as opções de visualização | menu da área vazia | lacuna |
| Trocar papel de parede | menu da área vazia | lacuna |
| Aparência do desktop | menu da área vazia | lacuna |
| Configurações de tela | menu da área vazia | lacuna |
| Abrir a lixeira pelo ícone | duplo clique, menu do ícone | lacuna |
| Esvaziar a lixeira pelo ícone | menu do ícone | lacuna |
| Ações de entrada `.desktop` | menu do lançador | lacuna |
| Pré-visualização em janela própria | `Ctrl+Espaço`, menu | lacuna |
| Copiar, copiar caminho, recortar, colar | `Ctrl+C`, `Ctrl+Shift+C`, `Ctrl+X`, `Ctrl+V` | lacuna |
| Renomear | `F2` | lacuna |
| Mover para a lixeira, excluir de vez | `Delete`, `Shift+Delete` | lacuna |
| Abrir em nova janela | `Shift+Enter` | lacuna |
| Menu de contexto dos itens | clique direito | lacuna |

## 4. Histórico aproveitado

Cenários de regressão tirados das descrições de cada PR, com os testes
existentes que os protegem.

### #14: menus de contexto, miniaturas e ícones nítidos

| Cenário | Testes existentes |
|---|---|
| Menus de arquivo, pasta e área vazia com criação, abrir com, cópia e movimentação, compactação, e-mail e propriedades | `domain/context_tests.rs`: `arquivo_e_pasta_expoem_operacoes_reais_e_propriedades`, `area_vazia_oferece_criacao_e_so_habilita_colar_com_clipboard`; `app/menu_actions_tests.rs`: todos os testes do arquivo |
| Miniaturas de PNG, JPEG e WebP | `domain/thumbnail_tests.rs`: `miniaturas_so_sao_pedidas_para_formatos_decodificaveis`; `app/previews_tests.rs`: `miniaturas_sao_geradas_somente_para_itens_visiveis` |
| Ícones do tema em alta resolução | `domain/icons_tests.rs`: `cada_tipo_tem_icone_de_tema_e_fallback_generico` |
| Alinhamento vertical do texto do menu | — (conferência visual) |

### #15: lixeira e restauração segura

| Cenário | Testes existentes |
|---|---|
| Lixeira na lateral, restaurar um item ou todos | `app/trash_tests.rs`: `restaura_somente_o_item_escolhido_pelo_menu`, `restaurar_todos_preserva_conflitos_e_restaura_os_demais` |
| Restaurar pelo caminho do `.trashinfo` sem sobrescrever | `adapters/home_trash_tests.rs`: `restaura_arquivo_e_pasta_para_o_caminho_original`, `conflito_no_destino_preserva_o_conteudo_e_o_registro_na_lixeira`, `recusa_item_sem_registro_e_nome_que_tenta_sair_da_lixeira` |
| Ações de edição bloqueadas dentro da lixeira | `app/trash_tests.rs`: `na_lixeira_teclas_de_apagar_e_criar_nao_alteram_itens` |
| Falhas parciais informadas | `app/trash_tests.rs`: `restaurar_todos_encerra_quando_o_pai_nao_pode_ser_recuperado` |

### #16: imagem no clipboard Wayland

| Cenário | Testes existentes |
|---|---|
| Uma imagem PNG, JPEG ou WebP é publicada com o MIME dela | `app/clipboard_tests.rs`: `ctrl_c_em_png_publica_imagem_e_preserva_copia_interna`, `jpeg_e_webp_publicam_os_formatos_correspondentes` |
| Seleção múltipla e arquivo comum seguem o fluxo interno | `app/clipboard_tests.rs`: `arquivo_comum_e_selecao_multipla_nao_publicam_imagem` |
| Falha externa mostra erro sem perder a cópia interna | `app/clipboard_tests.rs`: `falha_do_clipboard_externo_preserva_copia_interna_e_informa_erro` |

### #20: esvaziar a lixeira com confirmação

| Cenário | Testes existentes |
|---|---|
| Confirmação antes de excluir, inclusive itens ocultos pelo filtro | `app/empty_trash_tests.rs`: `esvaziar_exige_confirmacao_e_inclui_itens_ocultos` |
| Fora da lixeira não altera arquivos | `app/empty_trash_tests.rs`: `esvaziar_fora_da_lixeira_nao_altera_arquivos` |
| Falha libera os botões e mostra o erro | `app/empty_trash_tests.rs`: `falha_ao_esvaziar_libera_o_botao_e_exibe_o_erro` |
| Esvaziar arquivos, pastas e links sem seguir o destino | `adapters/home_trash_tests.rs`: `esvazia_arquivos_pastas_e_links_sem_seguir_o_destino`, `esvaziar_recusa_pasta_de_arquivos_redirecionada_por_link` |
| Lixeira vazia só avisa | — |

### #35: lateral e dispositivos

| Cenário | Testes existentes |
|---|---|
| Ícones simbólicos do tema na lateral | `domain/icons_tests.rs`: `sidebar_prefere_simbolico_e_preserva_fallback_do_tema` |
| Páginas de Recentes, Favoritos e Rede | `app/sidebar_tests.rs`: `favoritos_abrem_arquivo_e_navegacao_local_limpa_pagina_virtual`; `adapters/sidebar_source_tests.rs`: `recentes_ignoram_arquivo_removido_e_ordenam_pela_visita`, `rede_lista_arquivos_por_uri_gio` |
| Desmontar e ejetar separados | `app/volumes_eject_tests.rs`: `ejecao_e_aceita_so_para_volume_removivel_e_serializada`; `view/sidebar_tests.rs`: `ejetar_so_e_oferecido_para_unidade_removivel_de_bloco`, `montar_e_desmontar_continuam_com_indicacao_de_progresso` |
| Nome do aparelho sem protocolo MTP nem estado | `view/sidebar_tests.rs`: `telefone_mostra_nome_sem_tipo_ou_estado`, `disco_prioriza_o_nome_e_usa_capacidade_quando_sem_descricao` |

### #43: melhorias e arraste para outros aplicativos

| Cenário | Testes existentes |
|---|---|
| Local sem lixeira (celular, GVfs) oferece exclusão permanente | `app/removal_tests.rs`: `local_sem_lixeira_pede_exclusao_permanente_no_delete`; `adapters/local_disk_tests.rs`: `local_remoto_nao_tem_lixeira_e_mantem_o_arquivo`, `so_locais_fora_do_gvfs_tem_lixeira` |
| Exclusão permanente com `Shift+Delete` | `app/removal_tests.rs`: `shift_delete_abre_a_confirmacao_e_cancelar_mantem_o_item`, `enter_tambem_confirma_e_apaga_pastas_inteiras_fora_do_desfazer` |
| Ordenação por nome, tamanho, data e tipo, com pastas primeiro e ordem natural | `domain/order_tests.rs`: `padrao_e_nome_natural_com_pastas_antes`, `tamanho_ordena_arquivos_e_empata_pelo_nome`, `tipo_agrupa_pela_extensao_sem_extensao_primeiro`; `app/sort_tests.rs`: `a_ordem_vale_ao_navegar_e_ao_reler_a_pasta` |
| Extrair: um item vai direto, vários vão para uma pasta | `adapters/extract_tests.rs`: `zip_com_uma_pasta_no_topo_sai_direto_com_o_conteudo`, `varios_itens_no_topo_ganham_pasta_com_o_nome_do_arquivo_sem_sobrescrever` |
| Renomear em lote com `{n}` ou `a -> b`, prévia, cadeia e desfazer | `domain/batch_name_tests.rs`: `padrao_com_n_numera_na_ordem_e_mantem_a_extensao`, `seta_troca_um_trecho_em_todos_os_nomes`; `batch_rename_tests.rs`: `cadeia_de_nomes_nao_sobrescreve_ninguem`; `app/batch_tests.rs`: `ctrl_z_devolve_todos_os_nomes_de_uma_vez` |
| Busca recursiva com limite de resultados | `adapters/finder_tests.rs`: `para_no_limite_e_avisa` |
| FileManager1 | `adapters/file_manager1_tests.rs`: `mostrar_itens_abre_a_pasta_com_selecao_e_repassa_o_token` |
| Abas | `app/tabs_tests.rs`: `ctrl_t_abre_outra_aba_na_mesma_pasta_e_cada_uma_navega_sozinha` |
| Arrastar e soltar interno, `Ctrl` copia, desfazer | `app/drop_tests.rs`: `soltar_um_arquivo_sobre_uma_pasta_move_e_pode_desfazer`, `com_ctrl_pressionado_soltar_copia` |
| Arrastar para outros aplicativos | `app/handoff_tests.rs`: `sair_da_janela_arrastando_entrega_ao_compositor_uma_vez` |
| Abrir com reconhece o tipo pelo conteúdo | `adapters/xdg_open.rs`: `ini_com_texto_e_reconhecido_pelo_conteudo_como_texto` |
| Revisão: clipboard limpo só ao concluir o próprio recorte | `app/clipboard_files_tests.rs`: `mover_para_outra_pasta_nao_limpa_a_area_de_transferencia`, `recorte_concluido_nao_limpa_uma_selecao_mais_nova` |
| Revisão: lote atualiza os favoritos de uma vez | `app/batch_tests.rs`: `renomear_em_cadeia_mantem_cada_favorito_na_sua_pasta` |
| Revisão: resultados da busca com tamanho e data, também em aba guardada | `app/search_tests.rs`: `resultados_respeitam_a_ordem_por_tamanho_escolhida_antes_da_busca`; `app/tabs_tests.rs`: `a_busca_de_uma_aba_guardada_respeita_a_ordem_por_tamanho` |
| Revisão: desfazer com Substituir devolve o item substituído | `app/undo_tests.rs`: `desfazer_copia_que_substituiu_devolve_o_item_antigo`, `desfazer_movimento_que_substituiu_devolve_os_dois_itens` |
| Revisão: voltar a uma aba recalcula a rolagem | `app/tabs_tests.rs`: `voltar_a_uma_aba_ajusta_a_rolagem_ao_modo_de_exibicao_atual` |
| Revisão: extração consulta o cancelamento antes de publicar | `adapters/extract_tests.rs`: `cancelar_depois_do_ultimo_progresso_nao_publica_os_arquivos` |
| Revisão: arraste invalidado quando a pasta de origem muda | `app/drop_tests.rs`: `trocar_de_aba_durante_o_arraste_nao_solta_itens_da_outra_pasta` |
| Revisão: miniatura do cache confere o tamanho | `adapters/thumbnail_cache_tests.rs`: `miniatura_com_o_mesmo_segundo_e_outro_tamanho_e_refeita` |

### #45: marcadores e seletor de arquivos

| Cenário | Testes existentes |
|---|---|
| `Ctrl+D` marca a pasta selecionada | `app/bookmark_keys_tests.rs`: todos os testes do arquivo |
| Menu do marcador troca a cor e remove | `app/bookmark_menu_tests.rs`: todos os testes do arquivo |
| Marcadores num arquivo próprio do Sparky | `adapters/bookmark_file_tests.rs`: `o_arquivo_e_proprio_do_sparky_dentro_da_configuracao_do_usuario`, `gravar_e_ler_de_novo_preserva_a_ordem_e_o_formato` |
| Seletor pelo portal FileChooser abre e salva | `app/chooser/chooser_tests.rs`: todos os testes do arquivo; `adapters/portal/service_tests.rs`: `escolha_feita_responde_sucesso_com_as_uris` |
| `Close` do portal encerra a janela | `adapters/portal/dialog_tests.rs`: `fechar_o_pedido_encerra_a_janela_aberta` |
| Menu dos marcadores bloqueado na escolha | `app/chooser/chooser_tests.rs`: `na_escolha_menus_e_teclas_que_mexem_em_arquivos_nao_funcionam` |
