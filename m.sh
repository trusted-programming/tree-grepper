#!/bin/bash
target/debug/tree-grepper -q rust '(source_file)' $1 | sort | uniq 
d=$(dirname $1)/$(basename $1)
d=${d/.rs/}
for f in $d/*.rs.1; do
	sed -ie "s#+ <LIFETIME>\('.*\)</LIFETIME>#<LIFETIME>\1</LIFETIME>#g" $f
	sed -ie "s#<MUTABLE></MUTABLE>let #let <MUTABLE></MUTABLE> #g" $f
	sed -ie "s#<MUTABLE></MUTABLE>&#&<MUTABLE></MUTABLE>#g" $f
	sed -ie "s#<MUTABLE></MUTABLE>\* #\*<MUTABLE></MUTABLE> #g" $f
	sed -ie "s#<MUTABLE></MUTABLE>:#:<MUTABLE></MUTABLE> #g" $f
	rm -f "$f"e
done
