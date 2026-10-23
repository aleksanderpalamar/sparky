() {
  local file
  for file in /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh; do
    if [[ -r $file ]]; then
      source $file
      break
    fi
  done
  for file in /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh; do
    if [[ -r $file ]]; then
      source $file
      break
    fi
  done
}
