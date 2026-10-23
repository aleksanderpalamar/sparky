# Releases por tags

O workflow `.github/workflows/release.yml` gera e publica os pacotes quando o
mantenedor envia uma tag de versão. A tag deve seguir `vMAJOR.MINOR.PATCH`, com
três números sem zeros iniciais: `v1.0.0`, `v0.22.0` ou `v0.22.1`, por exemplo.
Pré-releases, sufixos de build e tags sem `v` não são aceitos nesta entrega.

## Preparar uma versão

1. Definir a versão conforme o impacto SemVer documentado no projeto.
2. Manter a mesma versão em `Cargo.toml`, nos crates locais do `Cargo.lock` e no
   `pkgver` de `packaging/arch/PKGBUILD`. O script propaga o `pkgrel` aos três
   formatos, incluindo wallpapers, e recusa valores que não sejam inteiros
   positivos canônicos. O valor atual é `1`.
3. Integrar as mudanças na `main` pelo PR e Quality Gate obrigatórios.
4. Criar a tag no commit integrado e enviar somente essa tag, quando a publicação
   da versão for autorizada. Não é necessário gerar os pacotes no computador local.

Exemplo para uma futura publicação da 0.22.0, depois de conferir a versão e o
commit da `main`:

```bash
git tag -a v0.22.0 <commit-integrado> -m 'Sparky 0.22.0'
git push origin v0.22.0
```

O workflow não cria tags, altera versões, faz commits ou merges. O envio da tag
é a ação que inicia o empacotamento e a publicação. A versão preparada nesta
branch é 0.22.0, conforme solicitado; a automação funciona também para os próximos
incrementos MAJOR, MINOR e PATCH, sem edição do workflow.

## Etapas e arquivos

A validação inicial exige formato canônico, versões coerentes, checkout no commit
da tag e ancestralidade em `origin/main`. Tags anotadas e leves são aceitas.
Tags malformadas que não correspondem ao filtro nem iniciam o workflow; aquelas
que passam pelo filtro amplo são recusadas pela validação estrita.

O Quality Gate existente é reutilizado por `workflow_call`, no mesmo commit da
tag: convenções, testes Python, build e testes Rust em debug/release, Clippy e
documentação. Somente depois do sucesso são iniciados dois builds independentes:

- Ubuntu 24.04: `packaging/build.sh deb` e `rpm`, usando Zig e a verificação
  existente do teto glibc 2.36. As ferramentas de empacotamento têm versão fixa.
- Container oficial Arch Linux `base-devel-20261018.0.612668`: instala as
  dependências declaradas pelo PKGBUILD e executa `packaging/build.sh pacman`
  como usuário sem privilégios. O makepkg compila e testa em release usando seu
  Rust stable. O source Git está fixado no commit do checkout, inclusive em tag
  histórica; atualizações posteriores da main não mudam essa origem.

São exigidos os seis pacotes principais e de wallpapers nos formatos `.deb`,
`.rpm` e `.pkg.tar.zst`, para x86-64. O pacote Arch de símbolos `sparky-debug`
também é incluído quando gerado pelo makepkg. A conferência recusa arquivos
inesperados, links, arquivos vazios e metadados de nome/versão/arquitetura
incompatíveis. O conjunto recebe `SHA256SUMS` e `release.json`, com versão,
revisão, commit de origem e hashes dos pacotes.

Os artefatos intermediários permanecem sete dias no Actions. Os anexos publicados
ficam na Release do repositório, com notas geradas pelo GitHub. Os pacotes não são
assinados com chaves de distribuição; SHA-256 verifica integridade e não substitui
uma assinatura. O workflow não instala nem inicia o Sparky.

## Permissões e novas tentativas

Todos os jobs têm somente `contents: read`, exceto o job final de publicação,
com `contents: write`. Não é necessário cadastrar um PAT ou segredo adicional:
a publicação usa o `GITHUB_TOKEN` fornecido ao job. As Actions oficiais são
fixadas por SHA e o checkout não persiste credenciais.

O job final verifica os hashes recebidos, cria um rascunho vinculado à tag já
existente, envia os anexos e só então publica. O destino atual da tag é consultado
diretamente no remoto, resolvendo tags anotadas até seu commit, antes da criação
e imediatamente antes da publicação. Ele deve coincidir com o commit original
do workflow e o registrado em release.json; tag removida ou movida impede a
publicação. Falha no envio deixa o rascunho para investigação.

Reexecutar o workflow pode retomar um rascunho. Anexos antigos ausentes do conjunto
local validado são removidos antes do upload, incluindo símbolos opcionais de
uma tentativa anterior. Depois do upload, o conjunto remoto deve corresponder
exatamente ao local; falhas de consulta ou remoção impedem a publicação. Uma Release
já publicada é recusada e não tem seus arquivos substituídos. Publicações da
mesma tag são serializadas. Um pacote corrigido deve receber a versão SemVer
adequada e uma nova tag, sem mover uma tag de versão publicada.

## Verificações locais

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s ci/tests -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s packaging -p 'test_*.py' -v
PYTHONDONTWRITEBYTECODE=1 python3 -m ci.check --base origin/main --worktree
bash -n ci/release-arch.sh ci/release-publish.sh packaging/build.sh packaging/arch/PKGBUILD
RUNNER_TEMP=/tmp/sparky-actionlint bash ci/actionlint.sh
```

Os testes de release usam repositórios e arquivos temporários, metadados simulados
e um `gh` falso. Não enviam tags, criam Releases remotas, instalam dependências no
host ou acessam a sessão gráfica. O teste de origem do Arch consulta somente o
Git e as declarações do PKGBUILD. O instalador `ci/release-arch.sh` é destinado
exclusivamente ao container efêmero do Actions.

Esses testes verificam as regras e os comandos da automação. A primeira execução
completa no GitHub e a instalação dos pacotes nas distribuições de destino são
validações adicionais; a aprovação local não comprova esses resultados.

Referências: [eventos de push](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#push),
[workflows reutilizáveis](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows),
[criação de Releases pelo gh](https://cli.github.com/manual/gh_release_create)
e [imagem oficial do Arch](https://hub.docker.com/_/archlinux).
