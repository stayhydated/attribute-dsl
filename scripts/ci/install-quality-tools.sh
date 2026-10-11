#!/usr/bin/env bash
set -euo pipefail

cargo binstall --locked --no-confirm taplo-cli@0.10.0 rumdl@0.2.78 cargo-sort-derives@0.13.0

tool_dir="${RUNNER_TEMP:?}/quality-tools"
mkdir -p "$tool_dir"
cd "$tool_dir"
curl --fail --location --retry 3 \
    https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_amd64.tar.gz \
    --output actionlint.tar.gz
printf '%s\n' '8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8  actionlint.tar.gz' | sha256sum --check
tar -xzf actionlint.tar.gz actionlint

curl --fail --location --retry 3 \
    https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.xz \
    --output shellcheck.tar.xz
printf '%s\n' '8c3be12b05d5c177a04c29e3c78ce89ac86f1595681cab149b65b97c4e227198  shellcheck.tar.xz' | sha256sum --check
tar -xJf shellcheck.tar.xz shellcheck-v0.11.0/shellcheck
mv shellcheck-v0.11.0/shellcheck shellcheck

curl --fail --location --retry 3 \
    https://github.com/crate-ci/typos/releases/download/v1.42.3/typos-v1.42.3-x86_64-unknown-linux-musl.tar.gz \
    --output typos.tar.gz
printf '%s\n' 'e083bf917d840563e77b35b64f43924d98fcb77acb69691455099d39e347d67d  typos.tar.gz' | sha256sum --check
tar -xzf typos.tar.gz ./typos
printf '%s\n' "$tool_dir" >> "${GITHUB_PATH:?}"
