#!/bin/bash
filename=$(basename -- "$fullfile")
extension="${filename##*.}"
filename="${filename%.*}"

mkeot $1 > $filename.eot
sfnt2woff $1
woff2_compress $1

