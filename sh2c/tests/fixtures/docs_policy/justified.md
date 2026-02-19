# Justified sh() usage

```sh2
# sh(...) because: demonstrates inline justification
sh("echo good")
```

Also with // comment style:

```sh2
// sh(...) because: process substitution not available as primitive
sh("diff <(sort a) <(sort b)")
```

And within 3 lines above:

```sh2
let x = "hello"
# sh(...) because: complex pipeline
let y = "world"
sh("echo complex")
```
