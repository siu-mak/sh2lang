#!/bin/sh
# This file contains only POSIX-compatible shell constructs

x=1

# Single-bracket test (POSIX)
if [ -n "$x" ]; then
    echo "x is set"
fi

# Function without local (POSIX)
func() {
    _x=1
    echo "$_x"
}

# Simple command
echo "hello world"

# POSIX-compatible string operations
y="${x:-default}"
echo "$y"
