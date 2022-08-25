#!/bin/bash
for f in *.opus; do
	ffmpeg -y -i $f ${f%.opus}.ogg
	rm $f
done