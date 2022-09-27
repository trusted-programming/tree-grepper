target/debug/tree-grepper -q rust "\
  (\
    [\
     (mutable_specifier) @m \
     (lifetime) @l \
     (unsafe_block) @ub \
     (block) @b \
     (function_modifiers) @uf \
     (function_item) @f \
    ]\
  )\
" $1 | awk '
BEGIN{
}
# unsafe block
/:ub:/ {
  split($0, a, /:/);
  file = a[1]
  row=a[2]
  col=a[3]
}
/:b:/ {
  split($0, a, /:/);
  if (a[1] == file && a[2] == row && a[3] > col) {
	  unsafe_block[a[1] " " a[2] " " a[3]] = 1
  } else {
          safe_block[a[1] " " a[2] " " a[3]] = 1
  }
}
# unsafe function
/:f:/ {
  split($0, a, /:/);
  file = a[1]
  row=a[2]
  col=a[3]
  functions[a[1] " " a[2] " " a[3]] = 1
}
/:uf:/ {
  split($0, a, /:/);
  if (a[1] == file && a[2] == row && a[3] > col) {
	  unsafe_functions[a[1] " " a[2] " " col] = 1
  }
}
# ownership
/:m:/{
  split($0, a, /:m:/);
  print a[1]
  split(a[1], b, /:/);
  print "OWNER" " " a[2] " " b[1] " " b[2] " " b[3]
}
# lifetime
/:l:/{
  split($0, a, /:l:/);
  print a[1]
  split(a[1], b, /:/);
  print "LIFETIME" " " a[2] " " b[1] " " b[2] " " b[3]
}
END {
   for (u in unsafe_block) {
      print "SAFENESS 0 " u;
   }
   for (block in safe_block) {
      print "SAFENESS 1 " block;
   }
   for (f in functions) {
      if (unsafe_functions[f] == 1) {
	      print "SAFENESS 0 " f;
      } else {
	      print "SAFENESS 1 " f;
      }
   }
}
' |& tee t.t
