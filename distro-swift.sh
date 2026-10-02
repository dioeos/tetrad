#!/usr/bin/env bash
set -euo pipefail

SWIFT_VERSION="6.4.0"

if ! command -v swiftly >/dev/null 2>&1; then
    curl -O "https://download.swift.org/swiftly/linux/swiftly-$(uname -m).tar.gz"
    tar zxf "swiftly-$(uname -m).tar.gz"

    ./swiftly init --quiet-shell-followup

    rm -f ./swiftly "swiftly-$(uname -m).tar.gz" LICENSE.txt
fi

. "${SWIFTLY_HOME_DIR:-$HOME/.local/share/swiftly}/env.sh"

if ! command -v swift >/dev/null 2>&1; then
    # swiftly install latest
    swiftly install "$SWIFT_VERSION"
fi

swift --version
sourcekit-lsp --version
