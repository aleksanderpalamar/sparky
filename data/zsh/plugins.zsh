autoload -Uz add-zsh-hook

copyfile() {
  emulate -L zsh
  if [[ -z $1 || ! -r $1 ]]; then
    print -u2 -- "copyfile: informe um arquivo legível"
    return 1
  fi
  wl-copy < "$1" && print -r -- "$1 copiado para a área de transferência"
}

sparky-copybuffer() {
  print -rn -- "$BUFFER" | wl-copy
}
zle -N sparky-copybuffer
bindkey '^O' sparky-copybuffer

typeset -ga sparky_dirs_back sparky_dirs_forward

sparky-dirs-remember() {
  sparky_dirs_back+=("$OLDPWD")
  sparky_dirs_forward=()
}
add-zsh-hook chpwd sparky-dirs-remember

sparky-dirs-redraw() {
  local hook
  for hook in $precmd_functions; do
    $hook
  done
  zle reset-prompt
}

sparky-dirs-back() {
  (( ${#sparky_dirs_back} )) || return 0
  sparky_dirs_forward+=("$PWD")
  builtin cd -q -- "${sparky_dirs_back[-1]}"
  sparky_dirs_back[-1]=()
  sparky-dirs-redraw
}

sparky-dirs-forward() {
  (( ${#sparky_dirs_forward} )) || return 0
  sparky_dirs_back+=("$PWD")
  builtin cd -q -- "${sparky_dirs_forward[-1]}"
  sparky_dirs_forward[-1]=()
  sparky-dirs-redraw
}

sparky-dirs-up() {
  builtin cd -- ..
  sparky-dirs-redraw
}

zle -N sparky-dirs-back
zle -N sparky-dirs-forward
zle -N sparky-dirs-up
bindkey '^[[1;3D' sparky-dirs-back
bindkey '^[[1;3C' sparky-dirs-forward
bindkey '^[[1;3A' sparky-dirs-up

() {
  local file
  for file in /usr/share/autojump/autojump.zsh /etc/profile.d/autojump.zsh /usr/share/autojump/autojump.sh; do
    if [[ -r $file ]]; then
      source $file
      return 0
    fi
  done
}

() {
  (( $+commands[fzf] )) || return 0
  local integration file
  integration="$(fzf --zsh 2>/dev/null)"
  if [[ -n $integration ]]; then
    eval "$integration"
    return 0
  fi
  for file in /usr/share/fzf/key-bindings.zsh /usr/share/fzf/completion.zsh /usr/share/doc/fzf/examples/key-bindings.zsh /usr/share/doc/fzf/examples/completion.zsh; do
    if [[ -r $file ]]; then
      source $file
    fi
  done
}
