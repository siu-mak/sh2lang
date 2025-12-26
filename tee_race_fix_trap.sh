#!/bin/bash
rm -f out.log
(
  # Trap EXIT to ensure we wait for tee even on exit
  trap 'exec >&-; exec 2>&-; wait' EXIT
  exec > >(tee out.log)
  echo "content with exit"
  exit 0
)
if [ -s out.log ]; then
  echo "FOUND"
else
  echo "EMPTY"
fi
cat out.log
