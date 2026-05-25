# pidnest Workflow Notes

pidnest is Linux-only. The workflows intentionally avoid macOS, Windows,
Android, JSON packaging, and CPU-level binary variants.

## CI

`.github/workflows/ci.yml` runs on pushes and pull requests targeting `main`.
It installs Rust 1.85.0 with `rustfmt` and `clippy`, restores the Rust cache,
installs `codespell`, and runs the local gate:

```sh
./check.sh
```

## Release

`.github/workflows/release.yml` runs when a tag matching `v*` is pushed. It
builds Linux release archives for:

- `linux-x86_64` with target `x86_64-unknown-linux-gnu`
- `linux-aarch64` with target `aarch64-unknown-linux-gnu`

Each archive contains a flat layout:

```text
pidnest
README.md
LICENSE
```

Release tags use normal semver first:

```text
v1.0.0
v1.0.1
```

Tags containing `-alpha.`, `-beta.`, or `-rc.` are marked as prereleases.
Tags like `vX.Y.Z` and `vX.Y.Z-stable.N` are normal releases.

## AUR Sync

`.github/workflows/aur.yml` syncs the `aur/pidnest-bin` package to:

```text
ssh://aur@aur.archlinux.org/pidnest-bin.git
```

It runs only after the release workflow sends an `aur-sync` dispatch event, or
when triggered manually with a tag. The required repository secret is:

```text
AUR_SSH_PRIVATE_KEY
```

The sync accepts `v1.0.0` or `1.0.0`, normalizes to `v1.0.0`, updates `pkgver`,
resets `pkgrel=1`, regenerates `.SRCINFO`, commits as `rezky_nightky`, and
pushes to the AUR `master` branch.

The release workflow dispatches AUR sync only for normal semver tags such as
`v1.0.0`. Prerelease tags and `vX.Y.Z-stable.N` releases do not sync to AUR yet.

## Release Flow

Run the local gate, commit, tag, and push the tag:

```sh
./check.sh
git add .
git commit -m "release: v1.0.0"
git tag -a v1.0.0 -m v1.0.0
git push origin main
git push origin v1.0.0
```
