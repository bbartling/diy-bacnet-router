#!/usr/bin/env bash
# One-time Buildroot lab setup for the ubuntu2 VM.
# Run on the VM after: ssh ben@127.0.0.1 -p 2222
set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/bbartling/diy-bacnet-router.git}"
REPO_DIR="${REPO_DIR:-$HOME/src/diy-bacnet-router}"
BRANCH="${BRANCH:-luna-max/m0-buildroot-ci-repair}"
BUILD_WORK_ROOT="${BUILD_WORK_ROOT:-$HOME/dbr-buildroot}"

echo "==> Installing Buildroot host packages (matches build-os.yml)"
sudo apt-get update
sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
  bc build-essential cpio file git libncurses-dev python3 rsync unzip wget xz-utils \
  curl ca-certificates e2fsprogs qemu-system-x86

if ! command -v node >/dev/null 2>&1 || [[ "$(node --version 2>/dev/null || true)" != v24* ]]; then
  echo "==> Installing Node.js 24"
  curl -fsSL https://deb.nodesource.com/setup_24.x | sudo -E bash -
  sudo DEBIAN_FRONTEND=noninteractive apt-get install --yes nodejs
fi

if ! command -v rustc >/dev/null 2>&1; then
  echo "==> Installing Rust 1.93.0 via rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.93.0
fi
# shellcheck disable=SC1091
source "$HOME/.cargo/env" 2>/dev/null || true
rustup default 1.93.0 2>/dev/null || true

echo "==> Tool versions"
git --version
node --version
npm --version
rustc --version
cargo --version
echo "CPUs: $(nproc)"
free -h | head -2
df -h / | tail -1

mkdir -p "$(dirname "$REPO_DIR")"
if [[ ! -d "$REPO_DIR/.git" ]]; then
  echo "==> Cloning $REPO_URL"
  git clone "$REPO_URL" "$REPO_DIR"
fi

cd "$REPO_DIR"
git fetch origin
git checkout "$BRANCH"
git pull --ff-only origin "$BRANCH" || true

echo "==> Repo at $(git rev-parse --short HEAD) on $(git branch --show-current)"
mkdir -p "$BUILD_WORK_ROOT"

echo "==> VM setup complete."
echo "    Build: cd $REPO_DIR && bash scripts/vm-build-x86.sh"
echo "    Refresh: cd $REPO_DIR && git pull --ff-only"
