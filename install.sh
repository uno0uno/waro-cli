#!/bin/sh
# waro-cli installer
# Usage: curl -fsSL https://raw.githubusercontent.com/uno0uno/waro-cli/main/install.sh | sh

set -e

REPO="uno0uno/waro-cli"
BINARY="waro"

# ── Helpers ──────────────────────────────────────────────────────────────────

say() { printf "\033[1m%s\033[0m\n" "$1"; }
err() { printf "\033[31merror:\033[0m %s\n" "$1" >&2; exit 1; }
warn() { printf "\033[33mwarn:\033[0m %s\n" "$1" >&2; }

# ── Detect downloader ─────────────────────────────────────────────────────────

if command -v curl > /dev/null 2>&1; then
    DOWNLOAD="curl -fsSL"
    DOWNLOAD_TO="curl -fsSL -o"
elif command -v wget > /dev/null 2>&1; then
    DOWNLOAD="wget -qO-"
    DOWNLOAD_TO="wget -qO"
else
    err "curl or wget is required. Install one and retry."
fi

# ── Detect OS and architecture ────────────────────────────────────────────────

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# macOS Apple Silicon reports arm64; normalize to aarch64
case "${ARCH}" in
    arm64) ARCH="aarch64" ;;
esac

case "${OS}-${ARCH}" in
    darwin-aarch64)  TARGET="aarch64-apple-darwin"      ;;
    darwin-x86_64)   TARGET="x86_64-apple-darwin"       ;;
    linux-aarch64)   TARGET="aarch64-unknown-linux-gnu" ;;
    linux-x86_64)    TARGET="x86_64-unknown-linux-gnu"  ;;
    *)
        err "Platform ${OS}-${ARCH} is not supported.
Supported platforms: macOS ARM64, macOS Intel, Linux x86_64, Linux ARM64
Open an issue: https://github.com/${REPO}/issues"
        ;;
esac

say "Detected platform: ${OS}-${ARCH} → ${TARGET}"

# ── Get latest release version ────────────────────────────────────────────────

say "Fetching latest release..."

API_URL="https://api.github.com/repos/${REPO}/releases/latest"
VERSION=$(${DOWNLOAD} "${API_URL}" \
    | grep '"tag_name"' \
    | head -1 \
    | sed 's/.*"tag_name": "\([^"]*\)".*/\1/')

if [ -z "${VERSION}" ]; then
    err "Could not determine latest version from GitHub API.
Either no release exists yet, or you hit the API rate limit (60 req/hr).
Try again later or check: https://github.com/${REPO}/releases"
fi

say "Latest version: ${VERSION}"

# ── Build download URLs ───────────────────────────────────────────────────────

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
ARCHIVE="waro-cli-${VERSION}-${TARGET}.tar.gz"
ARCHIVE_URL="${BASE_URL}/${ARCHIVE}"
CHECKSUMS_URL="${BASE_URL}/checksums.sha256"

# ── Download to temp dir ──────────────────────────────────────────────────────

tmp=$(mktemp -d)
trap 'rm -rf "${tmp}"' EXIT

say "Downloading ${ARCHIVE}..."
${DOWNLOAD_TO} "${tmp}/${ARCHIVE}" "${ARCHIVE_URL}" \
    || err "Download failed: ${ARCHIVE_URL}"

say "Downloading checksums..."
${DOWNLOAD_TO} "${tmp}/checksums.sha256" "${CHECKSUMS_URL}" \
    || err "Download failed: ${CHECKSUMS_URL}"

# ── Verify SHA256 ─────────────────────────────────────────────────────────────

say "Verifying checksum..."

if ! command -v openssl > /dev/null 2>&1; then
    warn "openssl not found — skipping checksum verification."
else
    COMPUTED=$(openssl dgst -sha256 -r "${tmp}/${ARCHIVE}" | awk '{print $1}')
    EXPECTED=$(grep "${ARCHIVE}" "${tmp}/checksums.sha256" | awk '{print $1}')

    if [ -z "${EXPECTED}" ]; then
        err "Checksum entry for ${ARCHIVE} not found in checksums.sha256"
    fi

    if [ "${COMPUTED}" != "${EXPECTED}" ]; then
        err "Checksum mismatch!
  Expected: ${EXPECTED}
  Got:      ${COMPUTED}
The download may be corrupted. Aborting."
    fi

    say "Checksum OK."
fi

# ── Extract binary ────────────────────────────────────────────────────────────

tar -xzf "${tmp}/${ARCHIVE}" -C "${tmp}"

if [ ! -f "${tmp}/${BINARY}" ]; then
    err "Binary '${BINARY}' not found in archive. This is a bug — please report it."
fi

chmod +x "${tmp}/${BINARY}"

# ── Install ───────────────────────────────────────────────────────────────────

INSTALL_DIR=""
NEEDS_SUDO=""

if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ -d "${HOME}/.local/bin" ] && [ -w "${HOME}/.local/bin" ]; then
    INSTALL_DIR="${HOME}/.local/bin"
elif mkdir -p "${HOME}/.local/bin" 2>/dev/null; then
    INSTALL_DIR="${HOME}/.local/bin"
else
    INSTALL_DIR="/usr/local/bin"
    NEEDS_SUDO="1"
fi

say "Installing to ${INSTALL_DIR}/${BINARY}..."

if [ -n "${NEEDS_SUDO}" ]; then
    sudo mv "${tmp}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
    mv "${tmp}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

# ── Verify installation ───────────────────────────────────────────────────────

if "${INSTALL_DIR}/${BINARY}" --version > /dev/null 2>&1; then
    say ""
    say "✓ waro-cli installed successfully!"
    "${INSTALL_DIR}/${BINARY}" --version
else
    warn "Binary installed but could not run '${BINARY} --version'."
fi

# ── PATH hint ─────────────────────────────────────────────────────────────────

case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;  # already in PATH
    *)
        printf "\n\033[33m[!]\033[0m Add %s to your PATH:\n" "${INSTALL_DIR}"
        printf "    echo 'export PATH=\"%s:\$PATH\"' >> ~/.zshrc\n" "${INSTALL_DIR}"
        printf "    source ~/.zshrc\n\n"
        ;;
esac

# ── Setup reminder ────────────────────────────────────────────────────────────

printf "\nNext step: set your API key\n"
printf "    export WARO_API_KEY=waro_sk_your_key_here\n"
printf "    waro --help\n\n"
