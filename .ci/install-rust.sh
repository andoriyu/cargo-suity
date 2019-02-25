#!/bin/env bash

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUSTUP_TOOLCHAIN
echo "##vso[task.setvariable variable=PATH;]$PATH:$HOME/.cargo/bin"
