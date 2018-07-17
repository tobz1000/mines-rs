#!/bin/bash

# Script to build server & gui, until I find out a better Cargo-based solution.
# [workspace] isn't suitable AFAIK because you can't specify different compile targets.

PROJECT_ROOT=$(pwd)

build_server() {
    echo "Build server"
    cd "${PROJECT_ROOT}" && cargo build
}

build_gui() {
    echo "Build GUI"
    install_cargo_web && \
    cd "${PROJECT_ROOT}/gui" && \
    cargo web build --target=wasm32-unknown-unknown
}

install_cargo_web() {
    cargo web -V &>/dev/null || (echo "Install cargo-web"; cargo install cargo-web)
}

build_server && build_gui