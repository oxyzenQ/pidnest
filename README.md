# pidnest

pidnest shows a clean process tree for a Linux user or UID.

## Usage

```sh
pidnest <USER_OR_UID>
pidnest --me
pidnest <USER_OR_UID> --live
pidnest <USER_OR_UID> --live --interval <SECONDS>
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
pidnest rezky --live --interval 6
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
pidnest rezky --live --interval 3
pidnest rezky --live --interval 60
```

Live mode footer:

```text
live mode · refresh 6s · press Ctrl+C to quit
```

Root examples:

```sh
pidnest root
sudo pidnest root
```

Version:

```text
pidnest v1.0.0
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

## License

MIT
