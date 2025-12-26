#!/bin/bash
rm -f out.log
(
  exec > >(tee out.log)
  echo "content"
  # Wait for process substitutions attached to this shell
  wait
)
if [ -s out.log ]; then
  echo "FOUND"
else
  echo "EMPTY"
fi
cat out.log
