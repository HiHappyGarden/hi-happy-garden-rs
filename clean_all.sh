#!/bin/bash

dirs=(
    "build"
    "cmake-build-debug"
    "cmake-build-release"
    "cmake-build-relwithdebinfo"
    "rust/target"
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