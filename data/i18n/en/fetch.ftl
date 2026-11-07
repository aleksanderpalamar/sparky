field-os = OS
field-kernel = Kernel
field-uptime = Uptime
field-cpu = CPU
field-gpu = GPU
field-memory = Memory
field-disk = Disk
field-terminal = Terminal
field-shell = Shell
field-packages = Packages
field-compositor = Compositor
field-session = Session
field-monitors = Monitors
field-resolution = Resolution
field-theme = Theme
field-icons = Icons
field-font = Font
field-colors = Colors
not-available = N/A
error-unknown-option = unknown option: { $option }
error-missing-value = missing value for { $option }
error-invalid-value = invalid value for { $option }: { $value }
error-unknown-field = unknown field: { $field }
error-config = configuration { $path }: { $error }
usage =
    Usage: sparky-fetch [options]
      --format visual|json|plain   chooses the output (--json and --plain are shortcuts)
      --color auto|always|never    uses ANSI colors (--no-color turns them off)
      --logo, --no-logo            shows or hides the logo
      --layout classic|stacked     logo beside or above the information
      --fields a,b,c               chooses the fields and their order
      --hide a,b                   hides fields
      --width N                    width in columns
      --timeout MS                 collection deadline in milliseconds
      --accent, --label-color, --value-color #rrggbb   changes the colors
      --config FILE, --no-config   uses another file or none
      --list-fields                lists the fields
      -h, --help                   shows this help
      -V, --version                shows the version
compositor-version = Sparky { $version }
session-real = Wayland · { $display }
session-nested = Nested Wayland · { $display }
session-unknown = Wayland · { $display }
theme-custom = Custom
