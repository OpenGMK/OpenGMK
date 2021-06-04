#!/usr/bin/env bash

# cargo install bindgen
bindgen "bindgen.h" \
    --allowlist-function '^(ig|Im).+' \
    --allowlist-type '^(ig|Im|STB_|Stb).+' \
    --ctypes-prefix '::chlorine' \
    --size_t-is-usize \
    --output "src/bindgen.rs" \
    --rustfmt-configuration-file "$(pwd)/.rustfmt.toml"
