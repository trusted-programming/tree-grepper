#cargo build
#target/debug/tree-grepper -q rust '(((function_item (identifier)@_i) @f (.eq? @_i "main")) (#sub! @f "unsafe @f" ))' src/main.rs |& tee t.t
#tree-sitter parse examples/insertion.rs
# tree-sitter parse examples/1.rs
# tree-sitter parse examples/1.rs | sed -e 's/ \[.*\])/)/g' -e 's/\[.*\]$//g' -e 's/$/\\/g'

target/debug/tree-grepper -q rust "\
  ((let_declaration \
    (mutable_specifier)\
    pattern: (identifier)\
    type: (scoped_type_identifier \
      path: (identifier)\
      name: (type_identifier))\
    value: (type_cast_expression \
      value: (integer_literal) @_i0 \
      type: (scoped_type_identifier \
        path: (identifier)\
        name: (type_identifier)))) @_l\
  (while_expression \
    condition: (binary_expression \
      left: (identifier)\
      right: (identifier))\
    body: (block)) @w \
  (#sub! @w \"for i in @_i0..n {}\") \
  ) \
" examples/1.rs |& tee t.t

target/debug/tree-grepper -q rust "(((function_item (identifier)@_i) @f\
(.eq? @_i \"main\")) (#sub! @f \"unsafe @f\" ))" src/main.rs |& tee t.t
