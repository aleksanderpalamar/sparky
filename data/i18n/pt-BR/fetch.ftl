field-os = Sistema
field-kernel = Kernel
field-uptime = Tempo ligado
field-cpu = CPU
field-gpu = GPU
field-memory = Memória
field-disk = Disco
field-terminal = Terminal
field-shell = Shell
field-packages = Pacotes
field-compositor = Compositor
field-session = Sessão
field-monitors = Monitores
field-resolution = Resolução
field-theme = Tema
field-icons = Ícones
field-font = Fonte
field-colors = Cores
not-available = N/A
error-unknown-option = opção desconhecida: { $option }
error-missing-value = falta o valor de { $option }
error-invalid-value = valor inválido para { $option }: { $value }
error-unknown-field = campo desconhecido: { $field }
error-config = configuração { $path }: { $error }
usage =
    Uso: sparky-fetch [opções]
      --format visual|json|plain   escolhe a saída (--json e --plain são atalhos)
      --color auto|always|never    usa cores ANSI (--no-color desliga)
      --logo, --no-logo            mostra ou esconde o logotipo
      --layout classic|stacked     logotipo ao lado ou acima das informações
      --fields a,b,c               escolhe os campos e a ordem
      --hide a,b                   esconde campos
      --width N                    largura em colunas
      --timeout MS                 prazo da coleta em milissegundos
      --accent, --label-color, --value-color #rrggbb   troca as cores
      --config ARQUIVO, --no-config   usa outro arquivo ou nenhum
      --list-fields                lista os campos
      -h, --help                   mostra esta ajuda
      -V, --version                mostra a versão
compositor-version = Sparky { $version }
session-real = Wayland · { $display }
session-nested = Wayland aninhado · { $display }
session-unknown = Wayland · { $display }
theme-custom = Personalizado
