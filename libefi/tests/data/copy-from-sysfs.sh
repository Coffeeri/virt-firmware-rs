#!/bin/sh

# args
name="$1"
dest="$2"

if test "$name" = "" -o "$dest" = ""; then
    echo "usage: $0 <varname> <destfile>"
    exit 1
fi

file=$(echo /sys/firmware/efi/efivars/${name}-*)
if test ! -f "$file"; then
    echo "not found: $file"
    exit 1
fi

dd if="$file" of="$dest" status=none bs=1 skip=4
echo "ok"
