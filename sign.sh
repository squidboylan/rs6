#!/bin/bash

size=$(stat -c%s bootblock)

if [ ${size} -gt 510 ] ; then
    echo "Bootblock is too large"
    exit 1
else
    diff=$(expr 510 - ${size})
    dd if=/dev/zero bs=1 count=${diff} >> bootblock
    echo -ne "\x55\xAA" >> bootblock
fi
