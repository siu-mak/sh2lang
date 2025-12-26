#!/bin/bash
case $- in *e*) __e=1;; *) __e=0;; esac; set +e; { sh -c 'printf hi'; } || true | sh -c 'exit 9' ; __sh2_status=$?; if [ "$__e" = 1 ]; then set -e; fi; :
echo
echo "Status: $__sh2_status"
