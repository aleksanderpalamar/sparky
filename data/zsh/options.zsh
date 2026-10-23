HISTFILE="${XDG_STATE_HOME:-$HOME/.local/state}/sparky/zsh_history"
[[ -d ${HISTFILE:h} ]] || mkdir -p -- ${HISTFILE:h}
HISTSIZE=50000
SAVEHIST=10000
setopt extended_history hist_expire_dups_first hist_ignore_dups hist_ignore_space hist_verify share_history
setopt auto_cd auto_pushd pushd_ignore_dups pushd_minus interactive_comments long_list_jobs multios
if [[ -z $LS_COLORS ]] && (( $+commands[dircolors] )); then
  eval "$(dircolors -b)"
fi
alias ls='ls --color=auto'
alias grep='grep --color=auto'
alias l='ls -lah'
alias la='ls -lAh'
alias ll='ls -lh'
alias lsa='ls -lah'
alias md='mkdir -p'
alias rd='rmdir'
alias -- -='cd -'
alias -g ...='../..'
alias -g ....='../../..'
alias -g .....='../../../..'
