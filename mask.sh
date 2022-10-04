#!/bin/bash
#
# Turning Rust code into masked sequences of tokens, which will be ready for Masked Language Model learning.
#
# From a Rust code file, we aim to obtain two sequences, one is an extension of the original source code by inserting special tokens 
# as the placeholders of their corresponding syntatical roles. 
#
# These include:
# - OWNERSHIP:  when a variable is introduced by let_declaration, or type_parameters, it can be declared by `mutable_specifiers`` as mutable. 
#   When such a mutable_specifier is absent, it is immutable by default. Hence, every OWNERSHIP placeholder could have either `mutable` or `immuntable` 
#   in their values.  
#
# - LIFETIME:  when a variable is introduced by a type_parameter or a reference_type, it can be declared by `lifetime` annotation as being globally static, or 
#   locally relative to other variable's lifetimes. Hence, every LIFETIME placeholder could have multiple labels `'static`, `'a``, `'b`, etc.
#
# - SAFENESS: the 'unsafe' modifiers are introduced before function items as function modifiers, or before blocks as unsafe_blocks. 
#   When such syntactical elements are introduced while `unsafe` modifiers are not found, they are regarded as safe by default. 
#   Hence, every SAFENESS placeholder could have either `unsafe` or `safe` in their values.
#
# Note. We adapt the tree-sitter-rust's grammar.js definition as follows:
# - Introduce `unsafe` as a new non-terminal to avoid ambiguity with other function_modifiers such as `async`, `const`, etc.
# - Introduce `function_modifiers` field so that it is possible to use the negate field feature in a tree-sitter query
# - Update the `Cargo.toml` dependencies of tree-sitter to 0.20.9 so that the vendor generated ABI is compatible with the `tree-sitter-cli` command
#
# To be consistent with the convention of CodeBERT, we need to mark the following Rust code:

# ```rust
# let mut num = 5; 

# let r1 = &num as *const i32; 
# let r2 = &mut num as *mut i32;

# unsafe { 
#     println!("r1 is: {}", *r1); 
#     println!("r2 is: {}", *r2); 
# }
# ```
# 
# as such:
# 
# ```rust
# let <MUTABLE>mut</MUTABLE> num = 5; 

# let r1 = &num as * <MUTABLE></MUTABLE>const i32; 
# let r2 = & <MUTABLE>mut</MUTABLE> num as *<MUTABLE>mut</MUTABLE> i32;

# <SAFENESS>unsafe</SAFENESS> { 
#     println!("r1 is: {}", *r1); 
#     println!("r2 is: {}", *r2); 
# }
# ```
# where the masks are introduced by XML markup elements.
#

function extract() {
	rm -f t.t
	target/debug/tree-grepper -q rust '([(_ safeness: _) @safe (function_item ! function_modifiers) @safe (unsafe_block) @ub ((function_modifiers (safeness)) . _ ) @uf (reference_type) @ref (type_arguments) @tref (lifetime) @lt (static_item) @mss (let_declaration) @mss (self_parameter) @mss (parameter) @mss (pointer_type) @mss (reference_expression) @mss (field_pattern) @mss (mut_pattern) @mss (reference_pattern) @mss (mutable_specifier) @ms ])' $1 | gawk '
BEGIN{
}
# unsafe block
/:ub:/ {
  split($0, a, /:/);
  file=a[1]
  s_byte = 0 + a[2]
  e_byte = 0 + a[5]
  unsafe_blocks[s_byte] = e_byte;
  # print $0
}
# unsafe item
/:safe:/ || /:safe_item:/ {
  split($0, a, /:/);
  file=a[1]
  s_byte = 0 + a[2]
  e_byte = 0 + a[5]
  safe_items[s_byte] = e_byte;
  # print $0
}
# unsafe function
/:uf:/ {
  split($0, a, /:/);
  file=a[1]
  s_byte = 0 + a[2]
  e_byte = 0 + a[5]
  unsafe_functions[s_byte] = e_byte;
  # print $0
}
# reference_type or type_arguments
/:ref:/ || /:tref:/ {
  split($0, a, /:/);
  file=a[1]
  s_byte = 0 + a[2]
  e_byte = 0 + a[5]
  lifetimes[s_byte] = e_byte;
  # print $0
}
/:lt:/ {
  split($0, a, /:/);
  file=a[1]
  s_byte = 0 + a[2]
  e_byte = 0 + a[5]
  lt[s_byte] = e_byte;
  # print $0
}

# mutable specifiers
/:mss:/ || /:ref:/ {
  split($0, a, /:/);
  file=a[1]
  s_byte = 0 + a[2]
  e_byte = 0 + a[5]
  mss[s_byte] = e_byte;
  # print $0
}
/:ms:/ {
  split($0, a, /:/);
  file=a[1]
  s_byte = 0 + a[2]
  e_byte = 0 + a[5]
  ms[s_byte] = e_byte;
  # print $0
}


END {
  file1 = file ".1";
  file2 = file ".2";
  system("rm -f " file1);
  system("rm -f " file2);
  asorti(unsafe_blocks, sorted_unsafe_blocks, "@ind_num_asc");
  L_UB = length(sorted_unsafe_blocks);
  asorti(unsafe_functions, sorted_unsafe_functions, "@ind_num_asc");
  L_UF = length(sorted_unsafe_functions);
  asorti(safe_items, sorted_safe_items, "@ind_num_asc");
  L_SI = length(sorted_safe_items);
  asorti(lifetimes, sorted_lifetimes, "@ind_num_asc");
  L_LTS = length(sorted_lifetimes);
  asorti(lt, sorted_lt, "@ind_num_asc");
  L_LT = length(sorted_lt);
  asorti(mss, sorted_mss, "@ind_num_asc");
  L_MSS = length(sorted_mss);
  asorti(ms, sorted_ms, "@ind_num_asc");
  L_MS = length(sorted_ms);
  lines = "";
  while ((getline line < file) != EOF) {
     lines = lines line "\n" 
  }
  L = length(lines);
  i_ub = 0;
  i_uf = 0;
  i_si = 0;
  i_lt = 0;
  i_ms = 0;
  for (b = 1; b <= L; b++) {
	  c = substr(lines, b, 1);
	  if (i_ub < L_UB) {
		  s_b = 0 + sorted_unsafe_blocks[i_ub + 1];
		  e_b = 0 + unsafe_blocks[s_b];
		  if (s_b <= b && b <= e_b) {
			  printf("<SAFENESS>") > file1;
			  printf("0\n") > file2;
			  i_ub++;
		  }
	  }
	  if (i_uf < L_UF) {
		  s_b = 0 + sorted_unsafe_functions[i_uf + 1];
		  e_b = 0 + unsafe_functions[s_b];
		  if (s_b <= b - 1 && b - 1 <= e_b) {
			  printf("safeness ", s_b, b, e_b) > file1;
			  printf("0\n", s_b, b, e_b) > file2;
			  i_uf++;
		  }
	  }
	  if (i_si < L_SI) {
		  s_b = 0 + sorted_safe_items[i_si + 1];
		  e_b = 0 + safe_items[s_b];
		  if (s_b <= b - 1 && b - 1 <= e_b) {
			  printf("safeness ", s_b, b, e_b) > file1;
			  str = substr(lines, s_b, e_b - s_b + 1);
			  gsub(/^[ \r\n\t]+/, "", str)
			  if (index(str, "unsafe") == 1) {
				  printf("0\n", s_b, b, e_b) > file2;
			  } else {
				  printf("1\n", s_b, b, e_b) > file2;
			  }
			  i_si++;
		  }
	  }
	  if (i_lt < L_LTS) {
		  s_b = 0 + sorted_lifetimes[i_lt + 1];
		  e_b = 0 + lifetimes[s_b];
		  if (s_b <= b - 1 && b - 1 <= e_b) {
			  printf(" lifetime ", s_b, b, e_b) > file1;
			  str = substr(lines, s_b, e_b - s_b + 1);
			  gsub(/^[ \r\n\t]+/, "", str)
			  has_lifetime = 0
			  for (j=1; j<= L_LT; j++) {
				  slt = 0 + sorted_lt[j] 
				  if (s_b <= slt && slt <= e_b) {
					  has_lifetime = 1
					  elt = 0 + lt[slt]
					  lll = substr(lines, slt + 1, elt - slt);
				  }
			  }
			  if (has_lifetime) {
				  printf("%s\n", lll, s_b, b, e_b) > file2;
			  } else {
				  printf("0\n", s_b, b, e_b) > file2;
			  }
			  i_lt++;
		  }
	  }
	  if (i_ms < L_MSS) {
		  s_b = 0 + sorted_mss[i_ms + 1];
		  e_b = 0 + mss[s_b];
		  if (s_b <= b - 1 && b - 1 <= e_b) {
			  printf(" mutable ", s_b, b, e_b) > file1;
			  str = substr(lines, s_b, e_b - s_b + 1);
			  gsub(/^[ \r\n\t]+/, "", str)
			  has_mutable = 0
			  for (j=1; j<= L_MS; j++) {
				  sms = 0 + sorted_ms[j] 
				  if (s_b <= sms && sms <= e_b) {
					  has_mutable = 1
					  ems = 0 + ms[sms]
					  lll = substr(lines, sms + 1, ems - sms);
				  }
			  }
			  if (has_mutable) {
				  printf("%s\n", lll, s_b, b, e_b) > file2;
			  } else {
				  printf("0\n", s_b, b, e_b) > file2;
			  }
			  i_ms++;
		  }
	  }
	  printf("%c", c) > file1;
  }
}
' 
}
extract $1 
sed -ie 's#unsafe #<SAFENESS>unsafe</SAFENESS> #g' $1.1
sed -ie "s#lifetime \(.*\) + '\([a-z]*\)#<LIFETIME>\2</LIFETIME> \1#g" $1.1
sed -ie "s#lifetime \(.*\)'\([a-z]*\)#<LIFETIME>\2</LIFETIME> \1#g" $1.1
sed -ie "s#lifetime \(.*\)#<LIFETIME></LIFETIME> \1#g" $1.1
sed -ie "s#\(.*\):\(.*\)+ '\([a-z]*\)#\1:<LIFETIME>\3</LIFETIME>\2#g" $1.1
sed -ie "s#safeness \(.*\)#<SAFENESS></SAFENESS> \1#g" $1.1
sed -ie "s#mutable let #let mutable #g" $1.1
sed -ie "s#mutable \([*&]\)#\1 mutable #g" $1.1
sed -ie "s#mutable \([^:]*\): #\1: mutable #g" $1.1
sed -ie "s#mutable mut#mut#g" $1.1
sed -ie "s#mut #<MUTABLE>mut</MUTABLE> #g" $1.1
sed -ie "s#mutable #<MUTABLE></MUTABLE> #g" $1.1
