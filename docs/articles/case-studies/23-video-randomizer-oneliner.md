---
title: "Case study: the \"random video sampler\" one-liner that nobody wants to touch"
description: "We dissect a wild Bash one-liner that plays random video clips, then refactor it into readable sh2 you can actually review."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Case study: the "random video sampler" one-liner that nobody wants to touch

## The message

Someone posted this in Slack:

> "Here's a neat one-liner that randomly samples video clips with mpv. Just run it in a directory with videos."

```bash
L=5; while true; do; readarray -t paths < <(find . -type f -print | shuf -n 1); for i in "${!paths[@]}"; do; path=${paths[i]}; if ffprobe -i "$path" -show_entries format=duration -v quiet -of csv="p=0" > /dev/null; then; N=$(ffprobe -i "$path" -show_entries format=duration -v quiet -of csv="p=0"); D=${N%.*}; P=$((D / 100 * 25)); R=$((1 + RANDOM % D - P * 2)); S=$((P + RANDOM % R)); W=$((R / 4)); LEN=$((1 + RANDOM % L)); mpv "$path" --start="$S" --length="$LEN" --fs &> /dev/null; W=$(bc <<< "$LEN - 0.5"); sleep "$W"; unset 'paths[i]'; fi; done; done
```

You squint at it. You want to trust it. But you can't mentally simulate it.

What does `R=$((1 + RANDOM % D - P * 2))` even mean? What happens if `D` is zero? Is `$path` safe if it has spaces? Why is `W` defined twice? What's with the `bc <<<` thing?

This is the kind of one-liner that works until it doesn't—and when it breaks, no one wants to debug it.

## The same Bash one-liner, formatted for humans

Here is that exact same command, but formatted so you can actually read it. It still uses all the Bash-specific features (`readarray`, process substitution, `$RANDOM`, here-strings), but at least the indentation shows the flow:

```bash
#!/usr/bin/env bash

L=5  # max clip length (seconds)

while true; do
  # Pick ONE random file path under the current directory.
  readarray -t paths < <(
    find . -type f -print | shuf -n 1
  )

  for i in "${!paths[@]}"; do
    path=${paths[i]}

    if ffprobe -i "$path" \
        -show_entries format=duration \
        -v quiet \
        -of csv="p=0" \
        > /dev/null
    then
      N=$(
        ffprobe -i "$path" \
          -show_entries format=duration \
          -v quiet \
          -of csv="p=0"
      )

      D=${N%.*}
      P=$(( D / 100 * 25 ))
      R=$(( 1 + RANDOM % D - P * 2 ))
      S=$(( P + RANDOM % R ))
      W=$(( R / 4 ))
      LEN=$(( 1 + RANDOM % L ))

      mpv "$path" --start="$S" --length="$LEN" --fs &> /dev/null

      W=$(bc <<< "$LEN - 0.5")
      sleep "$W"

      unset 'paths[i]'
    fi
  done
done
```

Now we can compare three things fairly: (1) compressed one-liner Bash, (2) formatted Bash, and (3) a full sh2 refactor.

**What became clearer just by formatting:**
* The control flow is visible (loop -> readarray -> for loop -> if media).
* The redundancy is obvious (calling `ffprobe` twice with identical flags).
* The variable reuse (`W` reused for both wait time and sleep, inside/outside the loop) is easier to spot.

**What remains difficult even after formatting:**
* **Process substitution**: `< <(...)` is still visually confusing and Bash-specific.
* **Array indices**: `${paths[i]}`/`${!paths[@]}` syntax is dense.
* **Implicit failures**: If `find` fails, the loop continues silently. If `bc` isn't installed, it crashes.
* **Random math**: The `RANDOM` logic is still a sea of magic numbers and edge cases.
* **Silent redirects**: `&> /dev/null` still swallows errors that you might want to see (like permission denied).

---

## What it does (plain English)

The *core intent* is pretty clear:

> Forever: pick a random file under `.` → if `ffprobe` thinks it's a media file, get its duration → choose a random start near the middle-ish (avoid edges) → play a short random clip with `mpv` fullscreen → sleep until almost done → repeat.

It's hard to validate mentally because it's:

* pipelines + process substitution + arrays
* `ffprobe` called twice
* random math with edge cases (`D=0`, very short clips)
* mixed concerns (select file / validate media / compute clip / play / sleep)

---

## Why it's hard to review (the audit list)

Before trusting this command, a reviewer would have to verify:

| Concern | What to check |
|---------|---------------|
| **Process substitution** | `< <(find ... | shuf)` — Bash-only, not POSIX |
| **readarray** | Bash 4+. Loads output into array. What if there's only one file? |
| **Array indexing** | `${paths[i]}` and `${!paths[@]}` — is this correct? |
| **Quoting** | `"$path"` — safe for spaces? What about newlines? |
| **ffprobe runs twice** | Once to check, once to get duration. Wasteful. |
| **Integer division** | `D=${N%.*}` strips decimals. What if `N` is "3.14159"? |
| **Divide-by-zero** | `R=$((1 + RANDOM % D - P * 2))` — if `D <= 0`, crash. |
| **Modulo bias** | `RANDOM % D` has slight bias for non-power-of-2 `D`. |
| **Variable reuse** | `W` is defined twice with different meanings. |
| **bc here-string** | `bc <<< "$LEN - 0.5"` — Bash-only, subprocess for simple math. |
| **Silent redirection** | `&> /dev/null` hides all output and errors. |
| **Infinite loop** | No way to stop except Ctrl+C. No error handling. |
| **No logging** | If a file fails, you'd never know. |
| **All files tried** | Will probe binaries, huge files, etc. |

That's at least 14 things to mentally verify for a "neat one-liner."

---

## Refactor plan

To make this reviewable, we need to:

1. **Remove the double ffprobe**: duration is captured once.
2. **Eliminate Bashisms**: `readarray`, process substitution, `$RANDOM`, `${x%.*}`.
3. **Make edge cases explicit**: guard against `D <= 1`, empty duration.
4. **Make random math readable and guarded** so it can't go negative.
5. **Separate concerns**: pick file, probe duration, compute segment, play clip.

---

## The sh2 version

### Recommended: A readable `.sh2` file

```sh2
func rand_int(n) {
  # returns 0..n-1, assumes n >= 1
  let s = trim(capture(run("shuf", "-i", "0-" & (n - 1), "-n", "1"), allow_fail=true))
  if status() != 0 { return 0 }
  return int(s)
}

func main() {
  let L = 5  # max clip length in seconds

  while true {
    # Pick one random file path using native pipeline
    let path = trim(capture(
      run("find", ".", "-type", "f", "-print") | run("shuf", "-n", "1"),
      allow_fail=true
    ))
    if status() != 0 || path == "" { continue }

    # Probe duration once. If not media, ffprobe fails -> skip.
    let dur_s = trim(capture(
      run("ffprobe",
        "-v", "quiet",
        "-show_entries", "format=duration",
        "-of", "csv=p=0",
        "-i", path
      ),
      allow_fail=true
    ))
    if status() != 0 || dur_s == "" { continue }

    # Convert duration like "123.456" -> integer seconds by stripping decimals.
    let D_str = before(dur_s, ".")
    if D_str == "" { continue }
    
    let D = int(D_str)
    if D <= 1 { continue }

    # P = 25% of duration
    let P = D / 4
    if P < 1 { set P = 1 }

    # R = D - 2P (usable range)
    let R = D - (P * 2)
    if R < 1 { set R = 1 }

    let S = P + rand_int(R)
    let LEN = 1 + rand_int(L)

    # Run mpv, play clip
    run("mpv", path, "--start=" & S, "--length=" & LEN, "--fs", allow_fail=true)

    # Sleep ~LEN - 1 (approximate the bc line safely)
    if LEN > 1 {
      run("sleep", str(LEN - 1))
    }
  }
}
```

### What's better here

* **Native pipelines**: `run(...) | run(...)` is readable and safe.
* **No "sh -c" escape hatches**: Logic stays in sh2.
* **No Bashisms**: `readarray`, process substitution, `$RANDOM`, `${x%.*}` are gone.
* **Edge cases are explicit**: `D <= 1`, `P < 1`, empty duration guarded.
* Uses `shuf` as a portable RNG instead of `$RANDOM`.

---

### If you insist on a `sh2do` one-liner

I don't recommend it for something this big, but here's the same idea in a single command (still readable-ish):

```bash
sh2do 'let L=5;
while true {
  let path=trim(capture(run("find",".","-type","f","-print")|run("shuf","-n","1"), allow_fail=true));
  if path=="" { continue }

  let dur_s=trim(capture(run("ffprobe","-v","quiet","-show_entries","format=duration","-of","csv=p=0","-i",path), allow_fail=true));
  if status()!=0 or dur_s=="" { continue }

  let D=int(before(dur_s,"."));
  if D<=1 { continue }

  let P=D/4;
  if P<1 { set P=1 }
  let R=D-(P*2);
  if R<1 { set R=1 }

  let S=P + int(trim(capture(run("shuf","-i","0-"&(R-1),"-n","1"), allow_fail=true)));
  let LEN=1 + int(trim(capture(run("shuf","-i","0-"&(L-1),"-n","1"), allow_fail=true)));

  run("mpv", path, "--start=" & S, "--length=" & LEN, "--fs", allow_fail=true);
  if LEN>1 { run("sleep", str(LEN-1)) }
}'
```

---

## Honest comparison

### What got better

| Aspect | Original | sh2 version |
|--------|----------|-------------|
| **Readability** | One dense line | 50+ lines with structure |
| **Named variables** | `L`, `D`, `P`, `R`, `S`, `W` | `L`, `D`, `P`, `R`, `S`, `LEN` (with clear comments) |
| **Double ffprobe** | Runs twice | Runs once |
| **Edge cases** | Crashes on `D=0` | Guarded with `if D <= 1 { continue }` |
| **Random math** | Can go negative | Explicitly guarded |
| **Bashisms** | Uses `$RANDOM`, process sub | Uses portable `shuf` and native pipelines |

### What stayed the same

| Aspect | Notes |
|--------|-------|
| **ffprobe dependency** | Still calls ffprobe (no sh2 alternative) |
| **Infinite loop** | Still runs forever (but now clearly labeled) |
| **All files probed** | Still tries every file type (see note below) |

---

## Quick reality check

Your original line will try **every file** under `.` including binaries, huge files, etc. You might want to restrict `find` to media extensions for performance and fewer probes:

```sh2
let path = trim(capture(
  run("find", ".", "-type", "f", "(", "-name", "*.mp4", "-o", "-name", "*.mkv", ")") | run("shuf", "-n", "1"),
  allow_fail=true
))
```

> **Note:** The `lines()` iterator splits on newlines. If your filenames contain newlines (rare but possible), this script will break. Robust NUL-safe iteration requires features sh2 doesn't have yet.

---

## Improvements now easy to add

Once structured, these become straightforward:

### 1. Limit iterations
```sh2
let played = 0
let max_clips = 10
while played < max_clips { ... set played = played + 1 }
```

### 2. Add confirmation before fullscreen
```sh2
if confirm("Play " & path & "?", default=true) {
    run("mpv", ...)
}
```

### 3. Verbose mode
```sh2
if env.VERBOSE == "1" {
    print("Duration: " & D & "s, Start: " & S & "s, Length: " & LEN & "s")
}
```

### 4. Dry-run mode
```sh2
if env.DRY_RUN == "1" {
    print("[dry-run] Would play: " & path)
} else {
    run("mpv", ...)
}
```

### 5. Directory argument
```sh2
let dir = arg(1, ".")
let path = trim(capture(run("find", dir, "-type", "f") | run("shuf", "-n", "1"), allow_fail=true))
```

---

## Takeaway

The original one-liner is a marvel of compression. It's also a maintenance hazard. When it works, it's magic. When it breaks, it's a puzzle.

The moral isn't "Bash bad." Bash is great for quick interactive work and dense pipelines.

The moral is: **one-liners become liabilities when they grow, get shared, and need review.**

sh2 doesn't eliminate complexity—it makes it visible. Instead of simulating shell semantics, you read functions. Instead of hunting for quoting bugs, you see variables as values. Instead of guessing error behavior, you see `allow_fail=true` and `status()`.

That's the difference between "neat" and "maintainable."

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests
