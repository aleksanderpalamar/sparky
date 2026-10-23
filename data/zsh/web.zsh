sparky-uri() {
  emulate -L zsh
  local LC_ALL=C character encoded=
  for character in ${(s::)1}; do
    if [[ $character == [A-Za-z0-9.~_-] ]]; then
      encoded+=$character
    else
      encoded+=$(printf '%%%02X' "'$character")
    fi
  done
  print -rn -- $encoded
}

web_search() {
  emulate -L zsh
  local address=$1
  shift
  xdg-open "${address}$(sparky-uri "$*")" >/dev/null 2>&1 &!
}

alias google='web_search "https://www.google.com/search?q="'
alias ddg='web_search "https://duckduckgo.com/?q="'
alias github='web_search "https://github.com/search?q="'
alias youtube='web_search "https://www.youtube.com/results?search_query="'
alias wiki='web_search "https://pt.wikipedia.org/w/index.php?search="'
alias stackoverflow='web_search "https://stackoverflow.com/search?q="'
