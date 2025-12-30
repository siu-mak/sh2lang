#!/bin/bash
# This file contains multiple bash-only constructs for testing POSIX lints

# Double-bracket test (bash-only)
if [[ -n "$x" ]]; then
    echo "x is set"
fi

# Local variable (bash-only)
func() {
    local x=1
    echo "$x"
}

# Set pipefail (bash-only)
set -o pipefail

# Process substitution (bash-only)
diff <(echo "a") <(echo "b")

# Array syntax (bash-only)
arr=(one two three)
echo "${arr[@]}"

# Arithmetic command (bash-only)
((x = 1 + 2))

# Here-string (bash-only)
cat <<< "hello"
