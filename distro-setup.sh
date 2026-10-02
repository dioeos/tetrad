#!/usr/bin/env bash
set -euo pipefail

distrobox assemble create --file distrobox.ini

distrobox enter swift-env -- ./distro-swift.sh
