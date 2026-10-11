#!/usr/bin/env bash
set -euo pipefail

cargo sort-derives --check
taplo fmt --check
rumdl check .
typos
actionlint -shellcheck "$(command -v shellcheck)"
shellcheck scripts/ci/*.sh
