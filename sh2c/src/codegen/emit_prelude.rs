use super::{PreludeUsage, TargetShell};

pub(super) fn emit_prelude(target: TargetShell, usage: &PreludeUsage) -> String {
    let mut s = String::new();

    // Always emit __sh2_check for fail-fast behavior
    match target {
        TargetShell::Bash => {
            s.push_str("__sh2_check() { local s=\"$1\"; local loc=\"$2\"; local mode=\"$3\"; if (( s != 0 )); then if [[ \"$mode\" == \"return\" ]]; then return \"$s\"; else if [[ -n \"$loc\" ]]; then printf 'Error in %s\\n' \"$loc\" >&2; fi; exit \"$s\"; fi; fi; }\n");
        }
        TargetShell::Posix => {
            s.push_str("__sh2_check() { __sh2_s=\"$1\"; __sh2_l=\"$2\"; __sh2_m=\"$3\"; if [ \"$__sh2_s\" -ne 0 ]; then if [ \"$__sh2_m\" = \"return\" ]; then return \"$__sh2_s\"; fi; if [ -n \"$__sh2_l\" ]; then printf 'Error in %s\\n' \"$__sh2_l\" >&2; fi; exit \"$__sh2_s\"; fi; }\n");
        }
    }


    if usage.sh_probe {
        match target {
            TargetShell::Bash => {
                s.push_str("__sh2_sh_probe() { local cmd=\"$1\"; if bash -c \"$cmd\"; then __sh2_status=0; else __sh2_status=$?; fi; return 0; }\n");
            }
            TargetShell::Posix => {
                s.push_str("__sh2_sh_probe() { cmd=\"$1\"; if sh -c \"$cmd\"; then __sh2_status=0; else __sh2_status=$?; fi; return 0; }\n");
            }
        }
    }

    if usage.sh_probe_args {
        match target {
            TargetShell::Bash => {
                s.push_str("__sh2_sh_probe_args() { local cmd=\"$1\"; shift; if bash -c \"$cmd\" bash \"$@\"; then __sh2_status=0; else __sh2_status=$?; fi; return 0; }\n");
            }
            TargetShell::Posix => {
                s.push_str("__sh2_sh_probe_args() { cmd=\"$1\"; shift; if sh -c \"$cmd\" sh \"$@\"; then __sh2_status=0; else __sh2_status=$?; fi; return 0; }\n");
            }
        }
    }

    if usage.coalesce {
        s.push_str("__sh2_coalesce() { if [ -n \"$1\" ]; then printf '%s' \"$1\"; else printf '%s' \"$2\"; fi; }\n");
    }
    if usage.trim {
        s.push_str(r#"__sh2_trim() { awk -v s="$1" 'BEGIN { sub(/^[[:space:]]+/, "", s); sub(/[[:space:]]+$/, "", s); printf "%s", s }'; }
"#);
    }
    if usage.before {
        s.push_str(r#"__sh2_before() { awk -v s="$1" -v sep="$2" 'BEGIN { n=index(s, sep); if(n==0) printf "%s", s; else printf "%s", substr(s, 1, n-1) }'; }
"#);
    }
    if usage.after {
        s.push_str(r#"__sh2_after() { awk -v s="$1" -v sep="$2" 'BEGIN { n=index(s, sep); if(n==0) printf ""; else printf "%s", substr(s, n+length(sep)) }'; }
"#);
    }
    if usage.replace {
        s.push_str(r#"__sh2_replace() { awk -v s="$1" -v old="$2" -v new="$3" 'BEGIN { if(old=="") { printf "%s", s; exit } len=length(old); while(i=index(s, old)) { printf "%s%s", substr(s, 1, i-1), new; s=substr(s, i+len) } printf "%s", s }'; }
"#);
    }
    if usage.split {
        match target {
            TargetShell::Bash => {
                s.push_str(r#"__sh2_split() {
  local -n __o=$1
  if [[ -z "$3" ]]; then eval "$1=(\"$2\")"; return; fi
  mapfile -t __o < <(awk -v s="$2" -v sep="$3" 'BEGIN {
     len=length(sep);
     while(i=index(s, sep)) { print substr(s, 1, i-1); s=substr(s, i+len) }
     print s
  }')
}
"#);
            }
            TargetShell::Posix => {
                s.push_str(r#"__sh2_tmpfiles=""
__sh2_tmpfile() {
    t=$(mktemp) || exit 1
    __sh2_tmpfiles="$__sh2_tmpfiles $t"
    echo "$t"
}
__sh2_cleanup_tmpfiles() {
    for t in $__sh2_tmpfiles; do rm -f "$t"; done
}
trap __sh2_cleanup_tmpfiles EXIT
__sh2_split() {
  awk -v s="$1" -v sep="$2" 'BEGIN {
     if(sep=="") { print s; exit }
     len=length(sep);
     while(i=index(s, sep)) { print substr(s, 1, i-1); s=substr(s, i+len) }
     print s
  }'
}
"#);
            }
        }
    }

    match target {
        TargetShell::Bash => {
            if usage.loc {
                s.push_str(r#"__sh2_err_handler() {
  local s=$?
  local loc="${__sh2_loc:-}"
  if [[ "${BASH_COMMAND}" == *"(exit "* ]]; then return $s; fi
  if (( ${__sh2_suppress_err_depth:-0} > 0 )); then return "$s"; fi
  if [[ -z "$loc" ]]; then return $s; fi
  if [[ "$loc" == "${__sh2_last_err_loc:-}" && "$s" == "${__sh2_last_err_status:-}" ]]; then return $s; fi
  __sh2_last_err_loc="$loc"
  __sh2_last_err_status="$s"
  printf "Error in %s\n" "$loc" >&2
  return $s
}
"#);

                s.push_str("set -o errtrace\n");
                s.push_str("trap '__sh2_err_handler' ERR\n");
            }
            if usage.matches {
                s.push_str("__sh2_matches() { [[ \"$1\" =~ $2 ]]; }\n");
            }
            if usage.parse_args {
                s.push_str(r#"__sh2_parse_args() {
  local out="" key val
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --) shift; while [ "$#" -gt 0 ]; do out="${out}P	${1}
"; shift; done; break ;;
      --*=*) key="${1%%=*}"; val="${1#*=}"; out="${out}F	${key}	${val}
" ;;
      --*) key="$1"; if [ "$#" -gt 1 ] && [ "${2}" != "--" ] && [[ ! "$2" =~ ^-- ]]; then val="$2"; shift; else val="true"; fi; out="${out}F	${key}	${val}
" ;;
      *) out="${out}P	${1}
" ;;
    esac
    shift
  done
  printf '%s' "$out"
}
"#);
            }
        }
        TargetShell::Posix => {
            if usage.matches {
                s.push_str(
                    r#"__sh2_matches() { printf '%s\n' "$1" | grep -Eq -- "$2"; }
"#,
                );
            }
            if usage.parse_args {
                s.push_str(r#"__sh2_parse_args() {
  __out="" 
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --) shift; while [ "$#" -gt 0 ]; do __out="${__out}P	${1}
"; shift; done; break ;;
      --*=*) __key="${1%%=*}"; __val="${1#*=}"; __out="${__out}F	${__key}	${__val}
" ;;
      --*) __key="$1"; __f=0; case "$2" in --*) __f=1;; esac
           if [ "$#" -gt 1 ] && [ "${2}" != "--" ] && [ "$__f" = 0 ]; then __val="$2"; shift; else __val="true"; fi
           __out="${__out}F	${__key}	${__val}
" ;;
      *) __out="${__out}P	${1}
" ;;
    esac
    shift
  done
  printf '%s' "$__out"
}
"#);
            }
        }
    }

    if usage.args_flags {
        s.push_str(
            r#"__sh2_args_flags() { printf '%s' "$1" | awk '/^F\t/ { sub(/^F\t/, ""); print }'; }
"#,
        );
    }
    if usage.args_positionals {
        s.push_str(r#"__sh2_args_positionals() { printf '%s' "$1" | awk '/^P\t/ { sub(/^P\t/, ""); print }'; }
"#);
    }
    if usage.args_flag_get {
        s.push_str(r#"__sh2_args_flag_get() { printf '%s' "$1" | awk -v k="$2" -F '\t' '{ if (sub(/^F\t/, "")) { if ($1==k) v=$2 } else if ($1==k) v=$2 } END { printf "%s", v }'; }
"#);
    }
    if usage.list_get {
        s.push_str(r#"__sh2_list_get() { printf '%s' "$1" | awk -v i="$2" 'NR==i+1 { printf "%s", $0; exit }'; }
"#);
    }
    if usage.load_envfile {
        s.push_str(r##"__sh2_load_envfile() { if [ -r "$1" ]; then awk '{ sub(/^[[:space:]]+/, ""); sub(/[[:space:]]+$/, ""); if($0=="" || substr($0,1,1)=="#") next; if(substr($0,1,7)=="export ") sub(/^export[[:space:]]+/, ""); idx=index($0,"="); if(idx==0) next; k=substr($0,1,idx-1); v=substr($0,idx+1); sub(/^[[:space:]]+/, "", k); sub(/[[:space:]]+$/, "", k); sub(/^[[:space:]]+/, "", v); sub(/[[:space:]]+$/, "", v); len=length(v); if(len>=2){ f=substr(v,1,1); l=substr(v,len,1); if((f=="\047" && l=="\047") || (f=="\"" && l=="\"")){ v=substr(v,2,len-2) } } printf "%s\t%s\n", k, v }' "$1" 2>/dev/null || true; fi; }
"##);
    }
    if usage.save_envfile {
        s.push_str(r#"__sh2_save_envfile() { printf '%s' "$2" | awk -F '\t' 'NF>=1{ print $1 "=" $2 }' > "$1"; }
"#);
    }
    if usage.json_kv {
        s.push_str(r#"__sh2_json_kv() { printf '%s' "$1" | awk -F '\t' 'function esc(s) { gsub(/\\/, "\\\\", s); gsub(/"/, "\\\"", s); gsub(/\t/, "\\t", s); gsub(/\r/, "\\r", s); gsub(/\n/, "\\n", s); return s; } { k=$1; v=$2; if (k == "") next; if (!(k in seen)) { ord[++n] = k; seen[k] = 1; } val[k] = v; } END { printf "{"; for (i=1; i<=n; i++) { k = ord[i]; v = val[k]; printf "%s\"%s\":\"%s\"", (i==1?"":","), esc(k), esc(v); } printf "}"; }'; }
"#);
    }
    if usage.which {
        s.push_str(
            r#"__sh2_which() {
  __sh2_cmd="$1"
  case "$__sh2_cmd" in
    */*)
      if [ -x "$__sh2_cmd" ] && [ ! -d "$__sh2_cmd" ]; then printf '%s' "$__sh2_cmd"; return 0; fi
      return 1
      ;;
  esac
  # Save IFS and globbing state
  __sh2_old_ifs="$IFS"
  case "$-" in *f*) __sh2_had_noglob=1;; *) __sh2_had_noglob=0;; esac
  IFS=:
  set -f
  # POSIX-safe PATH scan preserving empty segments
  __sh2_p="${PATH:-.}"
  __sh2_found=""
  while :; do
    case "$__sh2_p" in
      *:*)
        __sh2_d="${__sh2_p%%:*}"
        __sh2_p="${__sh2_p#*:}"
        ;;
      *)
        __sh2_d="$__sh2_p"
        __sh2_p=""
        ;;
    esac
    [ -z "$__sh2_d" ] && __sh2_d="."
    __sh2_t="$__sh2_d/$__sh2_cmd"
    if [ -x "$__sh2_t" ] && [ ! -d "$__sh2_t" ]; then
      __sh2_found="$__sh2_t"
      break
    fi
    [ -z "$__sh2_p" ] && break
  done
  # Restore IFS and globbing state
  IFS="$__sh2_old_ifs"
  if [ "$__sh2_had_noglob" = 1 ]; then set -f; else set +f; fi
  if [ -n "$__sh2_found" ]; then
    printf '%s' "$__sh2_found"
    return 0
  fi
  return 1
}

"#,
        );

    }
    if usage.require {
        s.push_str(r#"__sh2_require() { for c in "$@"; do if ! command -v -- "$c" >/dev/null 2>&1; then printf '%s\n' "missing required command: $c" >&2; exit 127; fi; done; }
"#);
    }
    if usage.tmpfile {
        s.push_str(r#"__sh2_tmpfile() { if command -v mktemp >/dev/null 2>&1; then mktemp; else printf "%s/sh2_tmp_%s_%s" "${TMPDIR:-/tmp}" "$$" "$(awk 'BEGIN{srand();print int(rand()*1000000)}')"; fi; }
"#);
    }
    if usage.find_files {
        match target {
            TargetShell::Bash => {
                s.push_str(r#"__sh2_find_files() {
  local __out_var="$1" __dir="$2" __pat="$3"
  local -n __ref="$__out_var"
  __ref=()
  while IFS= read -r -d '' __file; do
    __ref+=("$__file")
  done < <(find "$__dir" -name "$__pat" -print0 | LC_ALL=C sort -z)
}
"#);
            }
            TargetShell::Posix => {} // Compile error handled in emit_cmd
        }
    }
    if usage.read_file {
        s.push_str(
            r#"__sh2_read_file() { cat "$1"; }
"#,
        );
    }
    if usage.write_file {
        s.push_str(r#"__sh2_write_file() { if [ "$3" = "true" ]; then printf '%s' "$2" >> "$1"; else printf '%s' "$2" > "$1"; fi; }
"#);
    }
    if usage.lines {
        match target {
            TargetShell::Bash => {
                s.push_str(r#"__sh2_lines() { mapfile -t "$2" <<< "$1"; if [[ -z "$1" ]]; then eval "$2=()"; elif [[ "$1" == *$'\n' ]]; then eval "unset '$2[\${#$2[@]}-1]'"; fi; }
"#);
            }
            TargetShell::Posix => {
                // Not supported in POSIX sh
            }
        }
    }

    if usage.glob {
        match target {
            TargetShell::Bash => {
                // Use compgen -G for safe glob expansion (no eval of pattern)
                // Requires bash 4.3+ for local -n (nameref)
                // Note: compgen -G does not support -- separator
                s.push_str(r#"if [[ -z "${BASH_VERSINFO:-}" || ${BASH_VERSINFO[0]} -lt 4 || ( ${BASH_VERSINFO[0]} -eq 4 && ${BASH_VERSINFO[1]} -lt 3 ) ]]; then
  echo "sh2: glob() requires Bash 4.3+" >&2
  exit 2
fi
__sh2_glob() {
  local __out_var="$1" __pat="$2"
  if [[ -z "$__pat" ]]; then
    local -n __ref="$__out_var"
    __ref=()
    return 0
  fi
  local -a __tmp=() __sorted=()
  mapfile -t __tmp < <(compgen -G "$__pat" || true)
  if ((${#__tmp[@]})); then
    mapfile -t __sorted < <(printf '%s\n' "${__tmp[@]}" | LC_ALL=C sort)
  fi
  local -n __ref="$__out_var"
  __ref=("${__sorted[@]}")
}
"#);
            }
            TargetShell::Posix => {
                // Not supported in POSIX sh - compile error handled elsewhere
            }
        }
    }

    if usage.arg_dynamic {
        match target {
            TargetShell::Bash => {
                s.push_str("__sh2_arg_by_index() { local idx=\"$1\"; shift; ");
                s.push_str("if [[ ! \"$idx\" =~ ^[0-9]+$ ]] || (( idx < 1 )); then printf 'Error: arg(): index must be an integer >= 1\\n' >&2; kill -s TERM $$; exit 1; fi; ");
                s.push_str("if (( idx > $# )); then printf 'Error: arg(): index %s out of range (argc=%s)\\n' \"$idx\" \"$#\" >&2; kill -s TERM $$; exit 1; fi; ");
                s.push_str("printf '%s' \"${!idx}\"; ");
                s.push_str("}\n");
            }
            TargetShell::Posix => {
                s.push_str("__sh2_arg_by_index() { idx=\"$1\"; shift; ");
                s.push_str("case \"$idx\" in (''|*[!0-9]*) printf 'Error: arg(): index must be an integer >= 1\\n' >&2; kill -TERM $$; exit 1;; esac; ");
                s.push_str("if [ \"$idx\" -lt 1 ]; then printf 'Error: arg(): index must be an integer >= 1\\n' >&2; kill -TERM $$; exit 1; fi; ");
                s.push_str("if [ \"$idx\" -gt \"$#\" ]; then printf 'Error: arg(): index %s out of range (argc=%s)\\n' \"$idx\" \"$#\" >&2; kill -TERM $$; exit 1; fi; ");
                s.push_str("eval \"printf '%s' \\\"\\${$idx}\\\"\"; ");
                s.push_str("}\n");
            }
        }
    }

    if usage.contains {
        if target == TargetShell::Bash {
            s.push_str(r#"__sh2_contains() { local -n __arr="$1"; local __val="$2"; for __e in "${__arr[@]}"; do [[ "$__e" == "$__val" ]] && return 0; done; return 1; }
"#);
        }
    }
    if usage.starts_with {
         match target {
            TargetShell::Bash => {
                s.push_str(r#"__sh2_starts_with() { [[ "$1" == "$2"* ]]; return $?; }
"#);
            }
             TargetShell::Posix => {
                 s.push_str(r#"__sh2_starts_with() { case "$1" in "$2"*) return 0;; *) return 1;; esac; }
"#);
             }
         }
    }
    if usage.log {
        s.push_str(r#"__sh2_log_now() { if [ -n "${SH2_LOG_TS:-}" ]; then printf '%s' "$SH2_LOG_TS"; return 0; fi; date '+%Y-%m-%dT%H:%M:%S%z' 2>/dev/null || date 2>/dev/null || printf '%s' 'unknown-time'; }
"#);
        s.push_str(r#"__sh2_log() { if [ "$3" = "true" ]; then printf '%s\t%s\t%s\n' "$(__sh2_log_now)" "$1" "$2" >&2; else printf '%s\t%s\n' "$1" "$2" >&2; fi; }
"#);
    }
    if usage.home {
        s.push_str(
            r#"__sh2_home() { printf '%s' "${HOME-}"; }
"#,
        );
    }
    if usage.path_join {
        s.push_str(r#"__sh2_path_join() { out=''; for p in "$@"; do [ -z "$p" ] && continue; case "$p" in /*) out="$p";; *) if [ -z "$out" ]; then out="$p"; else while [ "${out%/}" != "$out" ]; do out="${out%/}"; done; while [ "${p#/}" != "$p" ]; do p="${p#/}"; done; out="${out}/${p}"; fi;; esac; done; printf '%s' "$out"; }
"#);
    }

    if usage.confirm {
        // __sh2_confirm prompt default "$@"
        // Prints "1" or "0" to stdout. Prompt goes to stderr.
        // Precedence: SH2_NO/SH2_YES > --no/--yes > CI/non-tty default > interactive prompt
        s.push_str(r#"__sh2_confirm() { __sh2_prompt="$1"; __sh2_default="$2"; shift 2; "#);
        // Env overrides (highest precedence)
        s.push_str(r#"if [ "${SH2_NO:-}" = "1" ]; then printf '%s' '0'; return 0; fi; "#);
        s.push_str(r#"if [ "${SH2_YES:-}" = "1" ]; then printf '%s' '1'; return 0; fi; "#);
        // Arg overrides
        s.push_str(r#"for __a in "$@"; do case "$__a" in --yes) printf '%s' '1'; return 0;; --no) printf '%s' '0'; return 0;; esac; done; "#);
        // Non-interactive: CI=true or stdin not a TTY
        s.push_str(r#"if [ "${CI:-}" = "true" ] || ! [ -t 0 ]; then printf '%s' "$__sh2_default"; return 0; fi; "#);
        // Interactive prompt loop
        s.push_str(r#"while true; do "#);
        s.push_str(r#"if [ "$__sh2_default" = "1" ]; then printf '%s [Y/n] ' "$__sh2_prompt" >&2; else printf '%s [y/N] ' "$__sh2_prompt" >&2; fi; "#);
        s.push_str(r#"if ! IFS= read -r __sh2_ans; then printf '%s' "$__sh2_default"; return 0; fi; "#);
        s.push_str(r#"__sh2_ans_lc="$(printf '%s' "$__sh2_ans" | tr '[:upper:]' '[:lower:]')"; "#);
        s.push_str(r#"case "$__sh2_ans_lc" in y|yes) printf '%s' '1'; return 0;; n|no) printf '%s' '0'; return 0;; '') printf '%s' "$__sh2_default"; return 0;; esac; "#);
        s.push_str("done; }\n");
    }

    if usage.uid {
        s.push_str("__sh2_uid=\"$(id -u 2>/dev/null || printf '%s' 0)\"\n");
    }
    s
}

pub(super) fn is_prelude_helper(name: &str) -> bool {
    crate::builtins::PRELUDE_HELPERS.contains(&name)
}
