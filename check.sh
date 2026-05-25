#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT_DIR"

PROJECT_NAME="pidnest"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
ok() { printf '  \033[32m✓\033[0m %s\n' "$*"; }
fail() { printf '  \033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }
step() { printf '\n\033[1m==>\033[0m %s\n' "$*"; }

run() {
  printf '  $ %s\n' "$*"
  "$@"
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing command: $1"
}

expect_fail_contains() {
  local name="$1"
  local expected="$2"
  shift 2

  local out="$TMP_DIR/${name}.out"

  set +e
  "$@" >"$out" 2>&1
  local status=$?
  set -e

  if [[ "$status" -eq 0 ]]; then
    cat "$out" >&2
    fail "$name: expected command to fail, but it succeeded"
  fi

  if ! grep -Fq -- "$expected" "$out"; then
    cat "$out" >&2
    fail "$name: expected output to contain: $expected"
  fi

  ok "$name"
}

assert_no_ansi() {
  local file="$1"
  if grep -q $'\033\[' "$file"; then
    cat "$file" >&2
    fail "ANSI escape codes found in $file"
  fi
}

assert_no_pid_text() {
  local file="$1"
  if grep -Eq ' pid=[0-9]+' "$file"; then
    cat "$file" >&2
    fail "--no-pid output still contains pid text"
  fi
}

bold "pidnest local quality gate"

step "Tooling"
need_cmd cargo
need_cmd rustc
need_cmd git
ok "required base commands found"

if ! cargo fmt --version >/dev/null 2>&1; then
  fail "rustfmt is missing. Run: rustup component add rustfmt"
fi
ok "rustfmt found"

if ! cargo clippy --version >/dev/null 2>&1; then
  fail "clippy is missing. Run: rustup component add clippy"
fi
ok "clippy found"

if [[ "${SKIP_CODESPELL:-0}" != "1" ]]; then
  command -v codespell >/dev/null 2>&1 || fail "codespell is missing. Install it on Arch Linux with: sudo pacman -S codespell"
  ok "codespell found"
else
  ok "codespell skipped by SKIP_CODESPELL=1"
fi

step "Rust version"
run rustc --version
run cargo --version

step "Formatting"
run cargo fmt --all -- --check
ok "format check passed"

step "Cargo check"
run cargo check --all-targets --all-features --locked
ok "cargo check passed"

step "Clippy"
run cargo clippy --all-targets --all-features --locked -- -D warnings
ok "clippy passed"

step "Tests"
run cargo test --all-targets --all-features --locked
ok "tests passed"

step "Release build"
run cargo build --release --locked
ok "release build passed"

BIN="$ROOT_DIR/target/release/$PROJECT_NAME"
[[ -x "$BIN" ]] || fail "release binary not found: $BIN"

step "Smoke tests"

run "$BIN" --version >"$TMP_DIR/version.out"
grep -Fq -- "pidnest" "$TMP_DIR/version.out" || fail "--version output does not mention pidnest"
ok "--version"

run "$BIN" --me --no-color >"$TMP_DIR/me_no_color.out"
grep -Fq -- "uid=" "$TMP_DIR/me_no_color.out" || fail "--me output does not contain uid="
assert_no_ansi "$TMP_DIR/me_no_color.out"
ok "--me --no-color"

run "$BIN" --me --no-pid --no-color >"$TMP_DIR/me_no_pid.out"
grep -Fq -- "uid=" "$TMP_DIR/me_no_pid.out" || fail "--me --no-pid output does not contain uid="
assert_no_ansi "$TMP_DIR/me_no_pid.out"
assert_no_pid_text "$TMP_DIR/me_no_pid.out"
ok "--me --no-pid --no-color"

expect_fail_contains \
  "unknown-user" \
  "unknown user" \
  "$BIN" "__pidnest_unknown_user_999999__"

expect_fail_contains \
  "interval-without-live" \
  "--interval requires --live" \
  "$BIN" "--interval" "6"

expect_fail_contains \
  "interval-too-low" \
  "--interval must be between 3 and 60 seconds" \
  "$BIN" "--me" "--interval" "2" "--live"

expect_fail_contains \
  "interval-too-high" \
  "--interval must be between 3 and 60 seconds" \
  "$BIN" "--me" "--interval" "61" "--live"

step "Codespell"
if [[ "${SKIP_CODESPELL:-0}" != "1" ]]; then
  run codespell --config .codespellrc .
  ok "codespell passed"
else
  ok "codespell skipped"
fi

step "Git status"
run git status -sb

printf '\n\033[32mAll checks passed.\033[0m Safe to commit.\n'
