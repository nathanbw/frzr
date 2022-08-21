#!/bin/sh
./target/debug/frzr init && ./target/debug/frzr dump | sed 's/"//g' >from-frzr.output && echo "output success"

sha256sum -c from-frzr.output && rm from-frzr.output

