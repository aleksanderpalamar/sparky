# Sparky Adaptive Resource Manager — Fase 0: diagnóstico

Referência técnica: `Sparky-Adaptive-Resource-Manager-plano-tecnico.md` (v1.0).
Data do diagnóstico: 2026-11-18. Commit analisado: `5996ac9` (`main`, v0.32.0).

## Branch

- Branch da feature: `feat/sparky-adaptive-resource-manager`.
- Branch-base: `main`. Todas as features e correções partem da `main` e voltam
  por PR com o Quality Gate. A `integration/sparky-next` é local, serve só para
  gerar pacotes de teste e tem o mesmo conteúdo da `main`.
- Trabalhos de outros agentes: não há branches remotas além da `main`. As
  worktrees em `dist/` estão em HEAD destacado e não foram tocadas.

## Ambiente de referência

| Item | Valor medido |
|---|---|
| Kernel | 7.3.4-arch1-1 |
| CPU | Ryzen 5 5500, 6 núcleos físicos, 12 threads, SMT irmãos `0,6`, L3 16 MiB único |
| Governor | `performance` |
| RAM | 15,4 GiB; swap zram de 4 GiB |
| GPU | Radeon RX 6600 (`amdgpu`, `card1`), 8 GiB de VRAM |
| Escalonadores de I/O | `none` e `mq-deadline` |
| `sched_ext` | disponível, desabilitado |

## Capacidades do Linux sem privilégio

Testadas com um processo descartável do próprio usuário.

| Interface | Resultado | Reversível sem privilégio |
|---|---|---|
| PSI cpu, memory, io (leitura) | disponível | — |
| Gatilhos PSI (`some 150ms/2s`) | aceitos nas três fontes | — |
| `sched_setaffinity` | reduzir e restaurar funcionou | sim |
| `ioprio` (`none`, best-effort 0–7, idle) | todas as trocas funcionaram, inclusive voltar a `none` | sim |
| `setpriority` (nice) | elevar funcionou; voltar a 0 deu `EACCES` | **não** (`RLIMIT_NICE` = 0) |
| amdgpu sysfs | `gpu_busy_percent`, `mem_busy_percent`, VRAM, `power1_average`, `temp1_input` | — |
| debugfs (DRM, sched) | negado | — |
| rtkit | presente, `MinNiceLevel` = -15 | só para threads que pedem para si |

### cgroups v2

- Controladores delegados ao `user@1000.service`: `cpu memory pids`. O
  controlador `io` **não** é delegado.
- O compositor e tudo o que ele lança (Steam, jogos, OBS, Discord iniciados
  pelo lançador) ficam em `session-3.scope`, criado pelo logind e de
  propriedade do root. O usuário não pode criar subgrupos nem mudar pesos ali.
- Só os serviços do `systemd --user` e os Flatpaks ficam em `app.slice`,
  onde o usuário tem delegação.

Consequência: hoje não há como redistribuir CPU por cgroup para jogos e OBS
sem privilégio ou sem mudar a forma como o compositor lança aplicativos.

## Referência inicial de desempenho

Sessão Sparky aberta, Steam em segundo plano, sem jogo, 348 processos e
1216 threads. Medido com Python em `nice 10` (o custo em Rust deve ser menor
e será medido na Fase 2).

| Leitura | p50 | p95 |
|---|---|---|
| `/proc/stat` | 26 µs | 32 µs |
| `/proc/meminfo` | 13 µs | 13 µs |
| PSI cpu + memory + io | 31 µs | 32 µs |
| amdgpu busy + VRAM | 27 µs | 34 µs |
| varredura de todos os `/proc/*/stat` | 5,0 ms | 5,6 ms |

- CPU total em 60 s: média de 2,31% e pico de 7,49% das 12 threads.
- PSI `some avg60`: cpu 0,00, memory 0,00, io 0,02.

Orçamento do plano em gaming: 0,1% de um núcleo, ou seja, 1 ms de CPU por
segundo. As leituras leves cabem nesse orçamento com folga a cada 2 s. Uma
varredura completa de processos não cabe se for feita a cada 2 s: em gaming,
a coleta por processo precisa se limitar aos processos de interesse, com
varredura completa espaçada.

## Componentes reutilizáveis

- **`sparky-ipc`**: socket Unix por sessão, JSON por linha, enums tipados,
  `Client` síncrono e assinatura de eventos (`WorkspacesChanged`, `Locked`,
  `Unlocked`).
- **`sparky-monitor`**: parsers de `/proc` e do DRM e o trait `SystemFiles`
  com fake para testes. Hoje estão dentro do binário do monitor.
- **`sparky-hardware`**: parsers puros de nome de CPU e GPU, sem topologia.
- **Escrita atômica**: o padrão completo (tmp, `sync_all`, rename, fsync do
  diretório, modo 600) existe só no `sparky-keyring`.
- **Interface**: iced 0.14, `sparky_ui::switch`, kit de linhas do settings,
  gráficos em canvas do monitor e `Catalog::for_domain` do `sparky-i18n`.
- **Configuração**: `sparky.toml` com `deny_unknown_fields`; uma seção nova
  exige entrar no schema do `sparky-config`.

## Lacunas

- **IPC**: não informa janela em foco, lista de janelas, app_id nem PID. Não
  tem versão de protocolo, limite de payload nem controle de assinantes lentos.
  O compositor já conhece app_id e consegue o PID do cliente Wayland.
- **Ciclo de vida**: os componentes da sessão são lançados sem supervisão nem
  reinício. Não há `.service` de usuário para daemons da sessão além do portal.
- **Telemetria**: o monitor não lê PSI, cgroups nem topologia da CPU.
- **Privilégios**: não existe helper root, `.policy` nem allowlist. O único uso
  de privilégio é `pkexec` direto no `sparky-store`.
- **Persistência**: não há crate de checkpoints nem uso de `XDG_STATE_HOME`
  além dos logs do compositor.
- **Interface**: não existe seção de desempenho no settings.

## Biblioteca neural

| Opção | Situação |
|---|---|
| Implementação própria em f32 | sem dependências, determinística, ~15 mil parâmetros na configuração inicial, treino com Adam abaixo de 1 MiB |
| Burn 0.22.1 (`flex` + `autodiff`) | tem GRU nativa; framework genérico e pesado para o tamanho do modelo; formato de checkpoint muda entre versões 0.x |
| Candle 0.11.1 | arrasta `tokenizers`, `zip` e `rayon` obrigatórios |
| tch-rs | exige libtorch em C++; descartado |
| dfdx, neuronika | sem release desde 2023 e 2022; descartados |

Recomendação: implementação própria, com gradient checking numérico nos testes
e Burn como plano B. Custos e latência ainda são estimativas e serão medidos
na Fase 4.

## Riscos

| Risco | Situação |
|---|---|
| Intervenções reversíveis limitadas | sem privilégio, só afinidade e prioridade de I/O; nice e cgroups dos apps dependem de decisão do mantenedor |
| Contexto gráfico | depende de estender o IPC do compositor |
| Orçamento em gaming | varredura completa de processos excede 0,1% de um núcleo a cada 2 s |
| Validação de gaming e OBS | exige sessões reais conduzidas pelo usuário |
| Testes de pressão de RAM | só em ambiente descartável |
| Telemetria de frametime e OBS | não há fonte confiável no sistema hoje |

## Decisões do mantenedor após o diagnóstico

| Tema | Decisão (2026-11-18) |
|---|---|
| Intervenções | Helper root com allowlist de operações tipadas e regra polkit |
| Contexto gráfico | Estender o `sparky-ipc` e o compositor com janelas, foco, app_id e PID, versão de protocolo e limite de payload |
| Ciclo de vida | Unit `systemd --user` presa ao `sparky-session.target`, com reinício, limites rígidos e watchdog |
| Rede neural | Implementação própria em f32, sem framework |

## Quality Gates

Todos os gates (QG01–QG21): **não executados**.

## Fases 1 e 2: resultados medidos

Medido em 2026-11-18 com o binário release, sem jogo aberto e sem o compositor
novo (o contexto gráfico ficou indisponível durante a medição).

| Medida | Resultado |
|---|---|
| Testes do crate `sparky-resources` | 75 aprovados |
| RSS do daemon | 3,1 MiB, pico 3,1 MiB |
| CPU do daemon em repouso (ciclo de 5 s, 300 s) | 0,030% de um núcleo |
| Amostra do sistema (p50 / p95) | 103 µs / 488 µs |
| Tabela incremental de processos (p50 / p95) | 186 µs / 208 µs |
| Varredura completa de processos (p50 / p95) | 3,5 ms / 3,6 ms |
| Reconhecimento de contexto (p50 / p95) | 9,6 µs / 10,3 µs |

Reprodução: `cargo run -p sparky-resources --release --example telemetry_cost`.

A primeira medição deu 0,100% por causa de varreduras completas a cada ciclo.
A tabela incremental lê só os processos novos e relê todos a cada 60 s.

Pendências: CPU no ciclo de 2 s durante jogos (QG07) e a precisão do
reconhecimento de contexto em sessões reais. As duas continuam **não validadas**.

Desvio registrado: "comunicação" é reconhecida, mas sozinha não suspende o
treinamento. Sem detectar uma chamada de voz, o Discord aberto o dia todo
impediria qualquer aprendizado.

## Fases 3 e 4: primeiro traço real

Traço `trace-desktop-1`: 20 min de desktop em 2026-11-18, amostra a cada
2 s, 600 linhas, sem jogo. Gravado com `--example record_trace`. Só métricas
agregadas, sem nomes de processos.

Baseline estatístico (`--example baseline_eval`), erro médio absoluto (MAE) em
pontos percentuais com horizonte de 10 s:

| Métrica | Persistência | Holt α0,5 β0,2 | SES α0,5 | SES α0,2 |
|---|---|---|---|---|
| CPU | 3,774 | 4,954 | 3,463 | 3,234 |
| Memória | 0,350 | 0,484 | 0,327 | 0,298 |
| GPU | 0,583 | 0,664 | 0,527 | 0,510 |
| PSI de I/O | 0,171 | 0,212 | 0,176 | 0,183 |

A tendência de Holt piorou todas as métricas neste uso. O baseline passou a ser
suavização exponencial simples (α = 0,2, sem tendência). Viés: α foi escolhido
neste mesmo traço; precisa ser reconfirmado com traços novos.

Comparação neural (`--example neural_eval`): 70% treino, 30% teste em ordem
temporal, 40 linhas de folga contra vazamento, semente fixa.

| Métrica (10 s) | Persistência | SES | GRU |
|---|---|---|---|
| CPU | 5,536 | 4,698 | 4,608 |
| Memória | 0,446 | 0,367 | 0,335 |
| GPU | 0,586 | 0,541 | 0,406 |
| PSI de I/O | 0,219 | 0,223 | 0,519 |
| VRAM | 0,044 | 0,047 | 0,789 |

- Parâmetros: GRU 5.166 + MLP 1.626 = 6.792 (limite 100 mil).
- Inferência GRU + MLP: p50 35 µs, p95 36 µs, p99 39 µs (meta 1 ms).
- Treino: 32 ms por época com 366 exemplos.

Conclusão: **inconclusivo**. A GRU ganhou em CPU, memória e GPU e perdeu nas
métricas próximas de zero. Com 20 min de dados não há base para a promoção. A
validação incremental exige vencer o baseline na média geral, então este
modelo seria recusado e o serviço continuaria no baseline. QG13 segue **não
validado**.

## Quality Gates em 2026-11-18

| Gate | Resultado | Evidência |
|---|---|---|
| QG01 Compilação | aprovado | `cargo test --workspace` compilou todos os crates |
| QG02 Qualidade | aprovado nos arquivos da branch | `cargo clippy --workspace --all-targets -D warnings` sem avisos; o `cargo fmt --check` acusa só arquivos do `sparky-bar` que a branch não toca |
| QG03 Regressão | aprovado | 2613 testes aprovados, 0 falhas, no workspace inteiro |
| QG04 Cobertura | aprovado | 164 testes no `sparky-resources`, além dos novos no IPC, config, monitor e settings |
| QG05 RAM normal | aprovado em repouso | RSS 3,1 MiB (meta 20 MiB) |
| QG06 RAM máxima | parcialmente | `MemoryMax=64M` na unit; pico em treino real não medido |
| QG07 CPU em jogo | não validado | 0,030% em repouso; falta sessão de jogo |
| QG08 GPU | aprovado | o modelo só usa CPU, sem dependência de GPU |
| QG09 Modelo | aprovado | 6.792 parâmetros (limite 100 mil) |
| QG10 Inferência | aprovado | p95 36 µs (meta 1 ms) |
| QG11 Jogo | não validado | depende de sessões do usuário |
| QG12 Jogo + OBS | não validado | depende de sessões do usuário |
| QG13 Utilidade neural | inconclusivo | 20 min de dados; GRU melhor em CPU, memória e GPU, pior perto de zero |
| QG14 Autorizações | aprovado nos testes | política nunca encerra nem suspende processos; só nice, ioprio e afinidade |
| QG15 Rollback | aprovado nos testes | restauração real em processo descartável; registro em tmpfs desfaz após queda e SIGTERM |
| QG16 Privacidade | aprovado | `RestrictAddressFamilies=AF_UNIX`; sem títulos de janela nem linha de comando |
| QG17 Persistência | aprovado nos testes | checkpoints com checksum, anterior como reserva, escrita atômica 600 |
| QG18 Robustez | aprovado nos testes | filas, histórico, registro de decisões e payload limitados |
| QG19 Isolamento | aprovado | o compositor só responde a pedidos; o daemon pode cair sem afetá-lo |
| QG20 Compatibilidade | parcialmente | GPU AMD validada; NVIDIA e Intel ficam como dado ausente |
| QG21 Integração | parcialmente | testes de renderização do Monitor e do Settings; validação visual pendente |

Pendências: helper root para restaurar nice (decidido, não implementado) e
comparações A/B/C/D em jogos reais.
