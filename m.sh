#!/bin/bash
target/debug/tree-grepper -q rust '(source_file)' $1 > $1.1
sed -ie "s#+ <LIFETIME>\('.*\)</LIFETIME>#<LIFETIME>\1</LIFETIME>#g" $1.1
sed -ie "s#<MUTABLE></MUTABLE>let #let <MUTABLE></MUTABLE> #g" $1.1
sed -ie "s#<MUTABLE></MUTABLE>&#&<MUTABLE></MUTABLE>#g" $1.1
sed -ie "s#<MUTABLE></MUTABLE>\* #\*<MUTABLE></MUTABLE> #g" $1.1
sed -ie "s#<MUTABLE></MUTABLE>:#:<MUTABLE></MUTABLE> #g" $1.1
