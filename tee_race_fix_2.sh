#!/bin/bash
rm -f out.log
(
  exec > >(tee out.log)
  echo "content"
  # Close stdout to signal tee to finish
  exec >&-
  wait
)
if [ -s out.log ]; then
  echo "FOUND"
else
  echo "EMPTY"
fi
cat out.log
