typeset -U path PATH
path=("$HOME/.local/bin" $path)
if [[ -d "$HOME/.cargo/bin" ]]; then
  path=("$HOME/.cargo/bin" $path)
fi
