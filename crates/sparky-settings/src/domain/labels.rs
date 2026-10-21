use crate::domain::field::Field;

impl Field {
    pub fn label(self) -> &'static str {
        match self {
            Field::GapsInner => "Folga entre janelas",
            Field::GapsOuter => "Folga da borda da tela",
            Field::Border => "Espessura da borda",
            Field::FloatWidth => "Largura das flutuantes",
            Field::FloatHeight => "Altura das flutuantes",
            Field::DefaultBinds => "Atalhos padrão do sparky",
            Field::Layout => "Layout",
            Field::Variant => "Variante",
            Field::Options => "Opções do XKB",
            Field::RepeatRate => "Repetições por segundo",
            Field::RepeatDelay => "Espera até repetir",
            Field::Terminal => "Terminal",
            Field::Launcher => "Lançador",
            Field::Files => "Gerenciador de arquivos",
            Field::Settings => "Configurações",
            Field::Screenshot => "Capturador de tela",
            Field::Locker => "Bloqueio de tela",
            Field::SystemMenu => "Menu do sistema",
            Field::Wallpaper => "Papel de parede",
        }
    }

    pub fn help(self) -> &'static str {
        match self {
            Field::GapsInner => "pixels de cada lado de cada janela",
            Field::GapsOuter => "pixels entre as janelas e a beira do monitor",
            Field::Border => "pixels; 0 tira a borda",
            Field::FloatWidth | Field::FloatHeight => "tamanho com que uma janela vira flutuante",
            Field::DefaultBinds => "desligado, valem só os atalhos escritos no sparky.toml",
            Field::Layout => "vazio usa o do sistema; vários separados por vírgula, como br,us",
            Field::Variant => "uma por layout, na mesma ordem, como abnt2",
            Field::Options => "como ctrl:nocaps ou grp:alt_shift_toggle",
            Field::RepeatRate => "quantas vezes por segundo a tecla segurada se repete",
            Field::RepeatDelay => "milissegundos segurando até começar a repetir",
            Field::Terminal => "vazio usa o primeiro terminal instalado",
            Field::Launcher => "SUPER + ESPAÇO",
            Field::Files => "SUPER + SHIFT + F",
            Field::Settings => "SUPER + ALT + ESPAÇO",
            Field::Screenshot => "PRINT e as variações",
            Field::Locker => "SUPER + CTRL + L; precisa falar ext-session-lock, como swaylock ou hyprlock",
            Field::SystemMenu => "SUPER + ESC; bloquear, suspender, sair, reiniciar e desligar",
            Field::Wallpaper => "caminho da imagem; vazio usa o do tema padrão; a seção Temas troca entre os do tema",
        }
    }
}
