sparky-git-status() {
  emulate -L zsh
  local -a lines fields
  lines=("${(@f)$(git status --porcelain=v2 --branch 2>/dev/null)}")
  [[ -n ${lines[1]} ]] || return 1
  local line branch= oid= sync= text
  local -i ahead=0 behind=0 staged=0 unstaged=0 untracked=0 conflicted=0 stashes=0
  for line in $lines; do
    case $line in
      ('# branch.head '*) branch=${line#\# branch.head } ;;
      ('# branch.oid '*) oid=${line#\# branch.oid } ;;
      ('# branch.ab '*)
        fields=(${=${line#\# branch.ab }})
        ahead=${fields[1]#+}
        behind=${fields[2]#-}
        ;;
      ([12]' '*)
        [[ ${line[3]} == . ]] || staged+=1
        [[ ${line[4]} == . ]] || unstaged+=1
        ;;
      ('u '*) conflicted+=1 ;;
      ('? '*) untracked+=1 ;;
    esac
  done
  [[ $branch == '(detached)' ]] && branch="@${oid[1,7]}"
  stashes=$(git rev-list --walk-reflogs --count refs/stash 2>/dev/null)
  (( behind )) && sync+="⇣$behind"
  (( ahead )) && sync+="⇡$ahead"
  text=$branch
  [[ -n $sync ]] && text+=" $sync"
  (( stashes )) && text+=" *$stashes"
  (( conflicted )) && text+=" ~$conflicted"
  (( staged )) && text+=" +$staged"
  (( unstaged )) && text+=" !$unstaged"
  (( untracked )) && text+=" ?$untracked"
  reply=(2 "$text")
  (( staged || unstaged || conflicted )) && reply[1]=3
  return 0
}
