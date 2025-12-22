#!/bin/bash

dirs=(
    "build"
    "hhg-app/target"
    "hhg-traits/target"
    "hhg-drivers/target"
    "osal-rs/osal-rs-build/target"
    "osal-rs/osal-rs-tests/target"
)

for dir in "${dirs[@]}"; do
    if [ -d "$dir" ]; then
        echo "Delete: $dir"
        rm -rf "$dir"
    else
        echo "Directory not find: $dir"
    fi
done

echo "Finish"