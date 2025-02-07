#!/bin/bash
# Copyright 2025, Giordano Salvador
# SPDX-License-Identifier: BSD-3-Clause

HOMEBREW_HOME=${HOMEBREW_HOME:=/opt/homebrew}
eval "$(${HOMEBREW_HOME}/bin/brew shellenv)"

brew install rustup

RUSTUP_CHANNEL=nightly-2025-01-30
RUSTUP_HOME=/root/.rustup

rustup toolchain install ${RUSTUP_CHANNEL}
rustup override set ${RUSTUP_CHANNEL}
rustup default ${RUSTUP_CHANNEL}

RUSTFLAGS=''
RUSTUP_TOOLCHAIN_PATH="$(rustup toolchain list | grep default | awk '{print $1}')"
RUSTUP_TOOLCHAIN="$(basename ${RUSTUP_TOOLCHAIN_PATH})"
RUST_SRC_PATH="${RUSTUP_TOOLCHAIN_PATH}/lib/rustlib/src/rust/src"
PATH=${RUSTUP_HOME}/toolchains/${RUSTUP_TOOLCHAIN}/bin${PATH:+:$PATH}
LD_LIBRARY_PATH=${RUSTUP_HOME}/toolchains/${RUSTUP_TOOLCHAIN}/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}

export HOMEBREW_HOME
export RUSTFLAGS
export RUST_SRC_PATH

case ${BUILD_MODE} in
    debug)      build_mode= ;;
    release)    build_mode=--release ;;
    *)          echo "Error: BUILD_MODE=$BUILD_MODE" >2  &&  exit 1 ;;
esac
