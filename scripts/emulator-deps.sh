#!/usr/bin/env bash
#
# Hi Happy Garden
# Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 2 of the License, or
# any later version.
#
# Fetches the C sources the emulator compiles (littlefs, cJSON) into
# build-emulator/_deps, at the same tags CMakeLists.txt uses for the firmware.
#
# Not needed after a firmware CMake build: main/build.rs finds them in
# build/_deps too. LITTLEFS_DIR / CJSON_SRC_DIR override both locations.

set -euo pipefail

# Keep in sync with the FetchContent_Declare tags in CMakeLists.txt
LITTLEFS_TAG="v2.11.2"
CJSON_TAG="v1.7.19"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPS="${ROOT}/build-emulator/_deps"

fetch() {
    local name="$1" url="$2" tag="$3"
    local dir="${DEPS}/${name}-src"

    if [[ -d "${dir}/.git" ]] && [[ "$(git -C "${dir}" describe --tags --exact-match 2>/dev/null)" == "${tag}" ]]; then
        echo "${name} ${tag} already in ${dir}"
        return
    fi

    rm -rf "${dir}"
    git -c advice.detachedHead=false clone --quiet --depth 1 --branch "${tag}" "${url}" "${dir}"
    echo "${name} ${tag} fetched into ${dir}"
}

mkdir -p "${DEPS}"
fetch littlefs https://github.com/littlefs-project/littlefs.git "${LITTLEFS_TAG}"
fetch cjson https://github.com/DaveGamble/cJSON.git "${CJSON_TAG}"
