zmodload zsh/datetime
autoload -Uz add-zsh-hook add-zle-hook-widget

typeset -g sparky_started= sparky_left= sparky_right= sparky_left_last=4 sparky_right_first=0
typeset -gi sparky_left_width=0 sparky_right_width=0

sparky-escape() {
  print -rn -- "${1//\%/%%}"
}

sparky-left-segment() {
  sparky_left+="%K{$1}%F{$2} $(sparky-escape "$3") "
  sparky_left_width+=$(( ${(m)#3} + 2 ))
  sparky_left_last=$1
}

sparky-right-segment() {
  [[ -n $sparky_right ]] || sparky_right_first=$1
  sparky_right+="%K{$1}%F{$2} $(sparky-escape "$3") "
  sparky_right_width+=$(( ${(m)#3} + 2 ))
}

sparky-elapsed() {
  local -i total=$1
  local text=
  (( total >= 86400 )) && text+="$(( total / 86400 ))d "
  (( total >= 3600 )) && text+="$(( total % 86400 / 3600 ))h "
  (( total >= 60 )) && text+="$(( total % 3600 / 60 ))m "
  print -rn -- "${text}$(( total % 60 ))s"
}

sparky-draw() {
  local -i gap=$(( COLUMNS - 1 - 2 - 3 - 3 - sparky_left_width - sparky_right_width ))
  local nothing= top="%F{8}╭─${sparky_left}%k%F{$sparky_left_last}▓▒░"
  if (( gap > 0 )); then
    top+="%F{8}${(pl:gap::·:)nothing}%F{$sparky_right_first}░▒▓${sparky_right}"
  fi
  PROMPT="${top}%k%f"$'\n'"%F{8}╰─%f "
}

sparky-precmd() {
  local -i code=$?
  local -a reply
  sparky_left= sparky_right= sparky_left_width=0 sparky_right_width=0
  sparky-left-segment 4 15 "${(%):-%~}"
  if sparky-git-status; then
    sparky-left-segment $reply[1] 0 "$reply[2]"
  fi
  if (( code )); then
    sparky-right-segment 1 3 "✘ $code"
  else
    sparky-right-segment 0 2 "✔"
  fi
  if [[ -n $sparky_started ]]; then
    local -i spent=$(( EPOCHREALTIME - sparky_started ))
    (( spent >= 3 )) && sparky-right-segment 3 0 "$(sparky-elapsed $spent)"
    sparky_started=
  fi
  (( ${#jobstates} )) && sparky-right-segment 0 6 $''
  sparky-right-segment 7 0 "$(strftime '%H:%M:%S' $EPOCHSECONDS)"
  sparky-draw
}

sparky-preexec() {
  sparky_started=$EPOCHREALTIME
}

sparky-transient() {
  PROMPT='%(?.%F{2}.%F{1})❯%f '
  zle .reset-prompt
}

TRAPWINCH() {
  sparky-draw
  zle && zle reset-prompt
}

add-zsh-hook precmd sparky-precmd
add-zsh-hook preexec sparky-preexec
add-zle-hook-widget line-finish sparky-transient
