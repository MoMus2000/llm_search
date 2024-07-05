#!/bin/bash

cargo build --release

sudo cp target/release/llm_search /usr/local/bin

echo "Consider setting up an alias in zsh.rc"
