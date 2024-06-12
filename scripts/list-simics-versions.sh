#!/bin/bash

ispm packages --list-available --json \
    | jq -r '.availablePackages[] | select(.pkgNumber == 1000) | .version' \
    | tac \
    | grep -v pre \
    | sed -n '/6.0.163/,$p'