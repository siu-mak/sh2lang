#!/bin/bash
rm -f out.log
(
  exec > >(tee out.log)
  echo "content"
)
# Intentionally no sleep
if [ -s out.log ]; then
  echo "FOUND"
else
  echo "EMPTY"
fi
cat out.log
