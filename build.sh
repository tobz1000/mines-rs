#!/bin/bash

# Convenience build script to install carg-web & build/run with necessary
# configuration for wasm target

PROJECT_ROOT=$(pwd)

if [ $# -gt 0 ] && [ $1 == "-s" ]; then
    CARGO_WEB_SUBCOMMAND="start"
else
    CARGO_WEB_SUBCOMMAND="build"
fi

install_cargo_web() {
    cargo web -V &>/dev/null || (set -o xtrace; cargo install cargo-web)
}

build() {
    set -o xtrace
    cargo web $CARGO_WEB_SUBCOMMAND --target=wasm32-unknown-unknown
}

install_cargo_web && build