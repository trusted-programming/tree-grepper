#!/bin/bash
if ! command -v tree-patcher >/dev/null 2>&1; then
	cargo install --git https://github.com/yijunyu/tree-grepper --branch patcher
fi
tokei --files --output=json | jq -r '.["Rust"].reports[].name' | xargs tree-patcher
