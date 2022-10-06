# tree-marker

Mark up Rust code for three type of masks which are prepared for Rust learning:

- Safeness: predicting `unsafe` keywords;

- Lifetime: predicting lifetime markers such as `'static`, `'a`, `'_`, etc.;

- Ownership: predicting `mut` keywords.

## Installation

Install directly from Git repository

```bash
cargo install --git https://github.com/yijunyu/tree-grepper --branch patcher
```

Install after checking out the Git repository

```bash
cd vendor/tree-sitter-rust
tree-sitter generate
tree-sitter parse ../../examples/error.rs
cd ../..
cargo build
```

## Usage

```bash
tree-marker foo.rs
```
which creates a folder `foo` with some numbered `N.rs.1` files where `N` is the offset of starting byte of a code fragment item.


When there are many Rust source code files, it is better to run the command in parallel:

```bash
tokei --files --output=json | jq -r '.["Rust"].reports[].name' | parallel --bar tree-marker
```

## License

See [LICENSE](./LICENSE) in the source.
