#!/bin/bash

function log_info {
    echo -e "\e[1m\e[4m\e[33mInfo: ${@}\e[39m\e[0m"
}


log_info "Updating stable Rust"
rustup update stable
log_info "Updating nightly Rust"
rustup update nightly

log_info "Checking code formatting..."
rustup run nightly cargo fmt -- --check || exit
log_info "Building..."
rustup run stable cargo build || exit
log_info "Running tests..."
rustup run stable cargo test || exit
