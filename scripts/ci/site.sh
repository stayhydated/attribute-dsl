#!/usr/bin/env bash
set -euo pipefail

export MDBOOK_BUILD__CREATE_MISSING=false
cargo run --package xtask --locked -- build book
cargo run --package xtask --locked -- build llms-txt
cargo run --package xtask --locked -- build web

# Exercise the same assembly as Pages, including book, llms and route fallbacks.
for output in index.html 404.html book/index.html llms.txt llms-full.txt sitemap.xml; do
    test -s "web/dist/$output"
done
test -f web/dist/.nojekyll
git diff --exit-code -- Cargo.lock
