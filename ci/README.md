# Quality Gate

O workflow `.github/workflows/quality-gate.yml` valida todos os PRs, sem filtros de
branch de destino ou de caminho. Também roda após pushes em main, na fila de
merge e por acionamento manual. Alterações na descrição do PR reavaliam o checklist.

O check obrigatório tem o nome estável **Quality Gate**. Ele exige sucesso do job
de convenções e das duas execuções Rust. Falhas, cancelamentos e jobs pulados
impedem sua aprovação. A configuração reproduzível de proteção está em
`.github/rulesets/quality-gate.json` e deve ser aplicada pelo administrador do
repositório, separadamente do token somente leitura do workflow.

## Verificações automáticas

| Verificação | Escopo |
|---|---|
| actionlint 1.7.12 | Sintaxe e semântica do workflow, com download verificado por SHA-256 |
| Testes do gate | Limites, eventos, caminhos, checklist, execução CLI e isolamento dos processos |
| Whitespace | Diff completo do PR |
| rustfmt | Arquivos Rust adicionados, modificados ou renomeados, sem percorrer módulos não alterados |
| Tamanho | Código novo até 200 linhas; arquivos antigos maiores podem manter ou reduzir o tamanho |
| Checklist | Seis declarações do modelo de PR, fora de exemplos em blocos de código ou comentários HTML |
| Branch | Branch de origem diferente da de destino |
| Build | Workspace em debug e release, com Cargo.lock obrigatório |
| Testes Rust | Unitários, integração e doctests em debug e release |
| Clippy | Workspace e todos os targets, com `-D warnings` |
| Documentação | Workspace sem dependências, com `RUSTDOCFLAGS=-D warnings` |
| Empacotamento | Testes Python e integração Rust: scripts, dependências, dados, binários, versão e licença de DEB/RPM/Arch |

O repositório já possuía divergências de rustfmt em arquivos antigos. O gate cobra
arquivos alterados pelo PR e preserva os demais. Ele não reforma o código existente.
O limite de linhas preserva renomeações sem ampliar a exceção histórica. Para
reduzir um arquivo grande, prefira módulos com responsabilidades coesas.

SRP, DIP, ausência de abstrações prematuras, condicionais, comentários autorizados,
tratamento de erros esperados, adequação dos testes e impacto SemVer exigem revisão
humana. O checklist registra essa avaliação e não é uma prova automática dessas
propriedades. Alterações de CI/documentação sem impacto no produto não exigem
incremento da versão dos pacotes.

O CI não valida interação com mouse/teclado real, DRM, PAM ou montagem de discos.
Essas evidências devem ser descritas no PR e obtidas somente com autorização.
Os pacotes não são gerados ou publicados em cada PR; o gate valida suas regras e
manifests. A geração e inspeção de artefatos de uma versão continuam no fluxo de
`packaging/build.sh`.

## Execução local

Requer Python 3.11+, Git, Rust 1.96.0 com rustfmt/Clippy e as bibliotecas Linux do
workflow. A validação dos arquivos considera o estado commitado por padrão.

```bash
python3 -m unittest discover -s ci/tests -v
python3 -m unittest discover -s packaging -p 'test_*.py' -v
python3 -m ci.check --base origin/main --worktree
bash -n ci/actionlint.sh ci/cargo.sh packaging/build.sh packaging/arch/PKGBUILD
bash ci/cargo.sh build --locked --workspace
bash ci/cargo.sh test --locked --workspace
bash ci/cargo.sh clippy --locked --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' bash ci/cargo.sh doc --locked --workspace --no-deps
RUNNER_TEMP=/tmp/sparky-actionlint bash ci/actionlint.sh
```

Acrescente `--release` a build/test para reproduzir a segunda execução. O wrapper
remove o ambiente gráfico e usa endereços D-Bus inexistentes; os testes de serviço
abrem apenas seus próprios sockets privados. Não execute um compositor ou agente
real para validar o gate.

O runner é Ubuntu 24.04 do GitHub, com duas tarefas de compilação e duas threads
de testes. As Actions são oficiais e fixadas por commit; o checkout não persiste
credenciais. Rust e actionlint têm versões fixas. Ao atualizar o Rust, revise a
versão instalada nos jobs, a chave do cache e este documento. Ao trocar o nome do
check final, atualize também o ruleset para evitar um check obrigatório pendente.
