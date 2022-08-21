#!/bin/sh
if ! [ -d ".frzr" ] ; then
  ./target/debug/frzr init
fi

./target/debug/frzr check && ./target/debug/frzr dump | sed 's/"//g' >from-frzr.output && echo "output success"

sha256sum -c from-frzr.output |grep -v -e 'OK$'

