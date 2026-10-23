() {
  local cache="${XDG_CACHE_HOME:-$HOME/.cache}/sparky"
  [[ -d $cache ]] || mkdir -p -- $cache
  zmodload zsh/complist
  autoload -Uz compinit
  compinit -i -d "$cache/zcompdump"
  zstyle ':completion:*' menu select
  zstyle ':completion:*' matcher-list 'm:{[:lower:][:upper:]}={[:upper:][:lower:]}' 'r:|=*' 'l:|=* r:|=*'
  zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
  zstyle ':completion:*' special-dirs true
  zstyle ':completion:*' use-cache yes
  zstyle ':completion:*' cache-path "$cache/zcompcache"
}
