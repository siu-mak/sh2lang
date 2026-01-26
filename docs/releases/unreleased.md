# Unreleased

## Added

### Pipe Blocks
Support for arbitrary statement blocks in pipelines:
- `pipe { ... } | { ... }`
- `run(...) | { ... }`
- `pipe { ... } | run(...)`
Mixed run/block stages are fully supported, with each stage running in an isolated subshell context.

### Pipeline Sudo
Pipelines now accept `sudo(...)` stages:
- `run("cmd") | sudo("cmd", n=true)`
- `pipe { ... } | sudo(...)`
`sudo` stages participate in the pipeline with correct pipefail and error handling, using the same options as standalone `sudo(...)`.
