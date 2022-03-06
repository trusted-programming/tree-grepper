cp -r ~/.tree-sitter/bin/tree-sitter-asm-0.0.1 vendor/
cargo build
cp target/debug/tree-grepper ~/.cargo/bin
