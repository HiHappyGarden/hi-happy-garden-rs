#!/bin/bash

dirs=(
    "build"
    "cmake-build*"
    "rust/target"
    "at-parser-rs/target"
    "osal-rs/target"
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