#!/bin/bash -e

if ! command -v "mdbook" > /dev/null 2>&1; then
    cargo install mdbook --no-default-features --features output,search,serve
fi

if ! command -v "mdbook-toc" > /dev/null 2>&1; then
    cargo install mdbook-toc mdbook-mermaid
fi

pushd doc/
mdbook build
popd