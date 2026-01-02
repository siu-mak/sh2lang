#!/bin/bash

f() {
    local x="local"
    echo "Direct: $x"
    echo "Unset+Printenv: $( ( unset x; printenv x ) 2>/dev/null || echo empty )"
}

echo "--- Run 4: Transient Environment ---"
x="outer" f
