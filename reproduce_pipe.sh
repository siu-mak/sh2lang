main() {
  if {
    printf "hi"
    {
       if [[ -o pipefail ]]; then __p=1; else __p=0; fi; set -o pipefail;
       sh -c 'exit 7' | sh -c 'cat'
       __sh2_status=$?
       if [ "$__p" = 0 ]; then set +o pipefail; fi
       (exit $__sh2_status)
    } &&
    printf "NO"
  }; then
    :
  else
    printf "CAUGHT\n"
    printf "%s\n" "$__sh2_status"
  fi
}
main
