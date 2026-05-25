# pidnest

pidnest shows a clean process tree for a Linux user or UID.

## Usage

```sh
pidnest <USER_OR_UID>
pidnest --me
pidnest -V
pidnest --version
```

Examples:

```sh
pidnest rezky
pidnest 1000
pidnest --me
```

Output:

```text
rezky uid=1000
└── bash pid=1234
    ├── python3 pid=1300
    └── cargo pid=1400
```

Version:

```text
pidnest v1.0.0
© 2026 rezky_nightky
MIT · github.com/oxyzenQ/pidnest
```

## Scope

`pidnest` scans `/proc`, reads `/proc/<pid>/status`, filters processes by UID,
and prints a stable parent-child tree. Processes that disappear or cannot be
read while scanning are skipped.

Live mode, colors, compact mode, and advanced TUI features are intentionally out
of scope for the MVP.

## Development

```sh
cargo fmt
cargo clippy
cargo test
```

## License

MIT
