#cargo build
#target/debug/tree-grepper -q rust '(((function_item (identifier)@_i) @f (.eq? @_i "main")) (#sub! @f "unsafe @f" ))' src/main.rs |& tee t.t
tree-sitter parse examples/insertion.rs
target/debug/tree-grepper -q rust '(((function_item (identifier)@_i) @f (.eq? @_i "main")) (#sub! @f "unsafe @f" ))' examples/insertion.rs |& tee t.t
