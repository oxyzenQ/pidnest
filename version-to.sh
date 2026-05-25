#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT_DIR"

fail() {
  printf 'version-to.sh: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat >&2 <<'EOF'
usage: ./version-to.sh vX.Y.Z
       ./version-to.sh X.Y.Z

Only normal semver releases are supported, such as v1.0.0 or 1.2.0.
EOF
}

[[ "$#" -eq 1 ]] || {
  usage
  exit 1
}

INPUT="$1"
if [[ "$INPUT" =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  VERSION="${INPUT#v}"
  TAG="$INPUT"
elif [[ "$INPUT" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  VERSION="$INPUT"
  TAG="v${INPUT}"
else
  usage
  fail "invalid version: $INPUT"
fi

required_files=(
  Cargo.toml
  Cargo.lock
  README.md
  workflow/about-ci.md
  aur/pidnest-bin/PKGBUILD
  aur/pidnest-bin/.SRCINFO
)

for file in "${required_files[@]}"; do
  [[ -f "$file" ]] || fail "required file is missing: $file"
done

command -v cargo >/dev/null 2>&1 || fail "cargo is required"
command -v makepkg >/dev/null 2>&1 || fail "makepkg is required to regenerate .SRCINFO"

printf 'Updating pidnest version to %s (%s)\n' "$VERSION" "$TAG"

VERSION="$VERSION" perl -0pi -e '
  my $version = $ENV{"VERSION"};
  s/(\[package\][\s\S]*?^version\s*=\s*)"[0-9]+\.[0-9]+\.[0-9]+"/${1}"$version"/m
' Cargo.toml

sed -i -E "s/^pkgver=.*/pkgver=${VERSION}/" aur/pidnest-bin/PKGBUILD
sed -i -E "s/^pkgrel=.*/pkgrel=1/" aur/pidnest-bin/PKGBUILD

TAG="$TAG" VERSION="$VERSION" perl -0pi -e '
  my $tag = $ENV{"TAG"};
  my $version = $ENV{"VERSION"};
  s/(pidnest v)[0-9]+\.[0-9]+\.[0-9]+/${1}$version/g;
  s|(releases/download/)v[0-9]+\.[0-9]+\.[0-9]+|${1}$tag|g;
  s/(pidnest-bin-)v[0-9]+\.[0-9]+\.[0-9]+(-linux-(?:x86_64|aarch64)\.tar\.gz)/${1}$tag$2/g;
' README.md

TAG="$TAG" VERSION="$VERSION" perl -0pi -e '
  my $tag = $ENV{"TAG"};
  my $version = $ENV{"VERSION"};
  s/The sync accepts `v[0-9]+\.[0-9]+\.[0-9]+` or `[0-9]+\.[0-9]+\.[0-9]+`, normalizes to `v[0-9]+\.[0-9]+\.[0-9]+`/The sync accepts `$tag` or `$version`, normalizes to `$tag`/g;
  s/`v[0-9]+\.[0-9]+\.[0-9]+`\. Prerelease tags/`$tag`. Prerelease tags/g;
  s/(git commit -m "release: )v[0-9]+\.[0-9]+\.[0-9]+(")/${1}$tag$2/g;
  s/(git tag -a )v[0-9]+\.[0-9]+\.[0-9]+( -m )v[0-9]+\.[0-9]+\.[0-9]+/${1}$tag$2$tag/g;
  s/(git push origin )v[0-9]+\.[0-9]+\.[0-9]+/${1}$tag/g;
' workflow/about-ci.md

cargo update -p pidnest >/dev/null

(
  cd aur/pidnest-bin
  makepkg --printsrcinfo > .SRCINFO
)

grep -Fq "version = \"${VERSION}\"" Cargo.toml \
  || fail "Cargo.toml does not contain version = \"${VERSION}\""
grep -Fq "pkgver=${VERSION}" aur/pidnest-bin/PKGBUILD \
  || fail "PKGBUILD does not contain pkgver=${VERSION}"
grep -Fq "pkgver = ${VERSION}" aur/pidnest-bin/.SRCINFO \
  || fail ".SRCINFO does not contain pkgver = ${VERSION}"
grep -Fq "$TAG" README.md \
  || fail "README.md does not contain ${TAG}"
grep -Fq "$TAG" workflow/about-ci.md \
  || fail "workflow/about-ci.md does not contain ${TAG}"

printf '\nVersion update summary:\n'
printf '  VERSION=%s\n' "$VERSION"
printf '  TAG=%s\n' "$TAG"
printf '  Cargo.toml package version updated\n'
printf '  Cargo.lock refreshed with cargo update -p pidnest\n'
printf '  AUR pkgver updated and pkgrel reset to 1\n'
printf '  AUR .SRCINFO regenerated with makepkg --printsrcinfo\n'
printf '  README.md and workflow/about-ci.md release examples updated\n'
