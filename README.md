<p align="center">
  <img src="assets/pidnest-logo.png" alt="pidnest logo" width="160">
</p>

<h1 align="center">pidnest</h1>

<p align="center">
  <a href="https://github.com/oxyzenQ/pidnest/actions/workflows/ci.yml"><img src="https://github.com/oxyzenQ/pidnest/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://ko-fi.com/rezky"><img src="https://img.shields.io/badge/Ko--fi-rezky-ff5f5f?logo=kofi&logoColor=white" alt="Ko-fi"></a>
</p>

<p align="center">pidnest shows a clean process tree for a Linux user or UID.</p>

## Usage

```sh
pidnest <USER_OR_UID>
pidnest --me
pidnest <USER_OR_UID> --live
pidnest <USER_OR_UID> --watch
pidnest <USER_OR_UID> --live --interval <SECONDS>
pidnest <USER_OR_UID> --depth <N>
pidnest <USER_OR_UID> --find <PATTERN>
pidnest <USER_OR_UID> --no-pid
pidnest <USER_OR_UID> --no-color
pidnest -V
pidnest --version
```

Examples:

```sh
pidnest rezky
pidnest 1000
pidnest --me
pidnest rezky --live
pidnest --me --watch
pidnest rezky --live --interval 6
pidnest --me --depth 2
pidnest --me --find codex
pidnest root --depth 1
pidnest rezky --no-pid
pidnest rezky --no-color
```

Normal output:

```text
rezky uid=1000
└── bash pid=1234
    ├── python3 pid=1300
    └── cargo pid=1400

3 roots · 18 processes
```

Without PID labels:

```text
rezky uid=1000
└── bash
    ├── python3
    └── cargo

3 roots · 18 processes
```

Live mode refreshes the tree in place. The default interval is 6 seconds. Custom
intervals must be between 3 and 60 seconds:

```sh
pidnest rezky --live
pidnest rezky --watch
pidnest rezky --live --interval 3
pidnest rezky --live --interval 60
```

Live mode footer:

```text
live mode · refresh 6s · press Ctrl+C to quit
```

Limit tree depth or find a process family:

```sh
pidnest --me --depth 2
pidnest --me --find codex
pidnest root --depth 1
```

Root examples:

```sh
pidnest root
sudo pidnest root
```

Version:

```sh
pidnest -V
pidnest --version
```

```text
pidnest v1.1.0 (<commit>)
© 2026 rezky_nightky
MIT · github.com/oxyzenQ/pidnest
```

## Behavior

`pidnest` scans `/proc`, reads `/proc/<pid>/status`, filters processes by UID,
and prints a stable parent-child tree. Processes that disappear or cannot be
read while scanning are skipped quietly.

Color output is automatic: enabled only when stdout is a TTY, disabled when
output is piped or redirected, and disabled when `NO_COLOR` is set. Use
`--no-color` to force plain output.

## Installation

Install from a GitHub release archive:

```sh
curl -fLO https://github.com/oxyzenQ/pidnest/releases/download/v1.1.0/pidnest-bin-v1.1.0-linux-x86_64.tar.gz
tar -xzf pidnest-bin-v1.1.0-linux-x86_64.tar.gz
sudo install -Dm755 pidnest /usr/local/bin/pidnest
```

For Linux aarch64:

```sh
curl -fLO https://github.com/oxyzenQ/pidnest/releases/download/v1.1.0/pidnest-bin-v1.1.0-linux-aarch64.tar.gz
tar -xzf pidnest-bin-v1.1.0-linux-aarch64.tar.gz
sudo install -Dm755 pidnest /usr/local/bin/pidnest
```

Install from the AUR:

```sh
yay -S pidnest-bin
```

## Development

MSRV: Rust 1.85+

```sh
cargo fmt
cargo clippy
cargo test
```

## Development Checks

Run the local quality gate before committing:

```sh
./check.sh
```

To skip the typo checker temporarily:

```sh
SKIP_CODESPELL=1 ./check.sh
```

Required tools:

- `rustfmt`
- `clippy`
- `codespell`

## Version Updates

Update release version references from one place:

```sh
./version-to.sh v1.2.0
```

The script accepts `vX.Y.Z` or `X.Y.Z`, updates Cargo metadata, README release
examples, workflow docs, and the AUR package metadata, then regenerates
`.SRCINFO` with `makepkg`.

Validate a version update by running:

```sh
./version-to.sh vX.Y.Z
./check.sh
```

## License

MIT
