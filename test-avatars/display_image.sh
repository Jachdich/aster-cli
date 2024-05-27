#!/bin/bash
# convert -scale 14x16 $1 - | kitty icat --place 2x1@10x10
cat $1 | kitty icat --place 2x1@12x27
