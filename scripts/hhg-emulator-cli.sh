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
# Client of the hhg-emulator control socket: sends one command and prints
# the reply, keeps the screen on view, or turns the keyboard into the
# encoder and the front panel button.
#
# Needs socat, or an nc with Unix socket support (-U -N).

set -euo pipefail

USAGE="\
Usage: $(basename "$0") [--control <socket>] <command> [args...]
       $(basename "$0") [--control <socket>] monitor [seconds]
       $(basename "$0") [--control <socket>] [--hold <ms>] auto

Options:
  --control <socket>  control socket path (default: \$XDG_RUNTIME_DIR/hhg-emulator.sock,
                      or /tmp/hhg-emulator.sock)
  --hold <ms>         auto: a key held longer than this is a long press, keep it above
                      the keyboard repeat delay (default: 700)
  -h, --help          print this help

Commands:
  monitor [seconds]   redraw the screen every [seconds], 0.5 by default (Ctrl+C to stop)
  auto                the keyboard drives the board (q or Ctrl+C to stop):
                        right / left          enc cw 1 / enc ccw 1
                        enter / held down     enc click / enc long
                        backspace / held down btn click / btn long
  anything else       sent to the emulator as it is, \"help\" lists the commands"

SOCKET="${XDG_RUNTIME_DIR:-/tmp}/hhg-emulator.sock"
HOLD_MS=700

# Once the command is sent the emulator answers and closes: wait for that,
# a long press keeps it busy for almost a second
if command -v socat >/dev/null; then
    connect() { socat -t 30 - "UNIX-CONNECT:${SOCKET}"; }
elif command -v nc >/dev/null; then
    connect() { nc -U -N "${SOCKET}"; }
else
    echo "socat or nc needed to reach the emulator" >&2
    exit 1
fi

# Sends one command line, the reply goes to stdout
send() {
    if [[ ! -S "${SOCKET}" ]]; then
        echo "no emulator on ${SOCKET}" >&2
        return 1
    fi
    printf '%s\n' "$*" | connect
}

# One command and its reply, failing on an ERR reply
command_mode() {
    # The emulator writes the image from its own directory, not from ours
    if [[ "${1:-}" == "display" && "${2:-}" == "pbm" && $# -eq 3 ]]; then
        set -- display pbm "$(realpath -m -- "$3")"
    fi

    local reply
    reply="$(send "$@")" || exit 1
    printf '%s\n' "${reply}"
    [[ "${reply}" != ERR* ]]
}

monitor_mode() {
    local interval="${1:-0.5}"
    exec watch -t -n "${interval}" -x "$0" --control "${SOCKET}" display
}

# Sends a command from auto mode, on one line with its reply
auto_send() {
    local reply
    reply="$(send "$@" 2>&1)" || true
    printf '%-12s %s\n' "$*" "${reply}"
}

# Reads one key into KEY, waiting at most $1 seconds when given: arrows come
# as an escape sequence, enter as a newline
read_key() {
    KEY=""
    if [[ $# -gt 0 ]]; then
        IFS= read -rsn1 -d '' -t "$1" KEY || return 1
    else
        IFS= read -rsn1 -d '' KEY || return 1
    fi
    if [[ "${KEY}" == $'\e' ]]; then
        local rest=""
        IFS= read -rsn2 -d '' -t 0.05 rest || true
        KEY+="${rest}"
    fi
}

# Enter or backspace: held down, the keyboard repeat sends it again before the
# hold time is over. A different key in the meantime ends a click, and is
# left in PENDING for the main loop.
press() {
    local key="$1" target="$2" hold
    hold="$(printf '%d.%03d' $((HOLD_MS / 1000)) $((HOLD_MS % 1000)))"

    if ! read_key "${hold}"; then
        auto_send "${target}" click
    elif [[ "${KEY}" == "${key}" ]]; then
        auto_send "${target}" long
        # The rest of the repeat, until the key is let go
        while read_key 0.15 && [[ "${KEY}" == "${key}" ]]; do :; done
        if [[ "${KEY}" != "${key}" ]]; then
            PENDING="${KEY}"
        fi
    else
        auto_send "${target}" click
        PENDING="${KEY}"
    fi
}

auto_mode() {
    if [[ ! -t 0 ]]; then
        echo "auto needs a terminal" >&2
        exit 1
    fi
    send status >/dev/null || exit 1

    # Global: the EXIT trap runs once this function is gone
    STTY_SAVED="$(stty -g)"
    trap 'stty "${STTY_SAVED}"' EXIT
    trap 'exit 0' INT TERM
    stty -echo -icanon min 1

    echo "auto on ${SOCKET}: right/left encoder, enter encoder button, backspace front button, q to quit"

    PENDING=""
    while true; do
        if [[ -n "${PENDING}" ]]; then
            KEY="${PENDING}"
            PENDING=""
        else
            read_key || break
        fi

        case "${KEY}" in
            $'\e[C' | $'\eOC') auto_send enc cw 1 ;;
            $'\e[D' | $'\eOD') auto_send enc ccw 1 ;;
            $'\n' | $'\r') press "${KEY}" enc ;;
            $'\x7f' | $'\b') press "${KEY}" btn ;;
            q | Q) break ;;
        esac
    done
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --control | --hold)
            if [[ $# -lt 2 ]]; then
                printf '%s needs a value\n\n%s\n' "$1" "${USAGE}" >&2
                exit 1
            fi
            if [[ "$1" == "--control" ]]; then
                SOCKET="$2"
            elif [[ "$2" =~ ^[0-9]+$ ]]; then
                HOLD_MS="$2"
            else
                echo "--hold needs milliseconds: $2" >&2
                exit 1
            fi
            shift 2
            ;;
        -h | --help)
            echo "${USAGE}"
            exit 0
            ;;
        *) break ;;
    esac
done

if [[ $# -eq 0 ]]; then
    echo "${USAGE}" >&2
    exit 1
fi

case "$1" in
    monitor) shift; monitor_mode "$@" ;;
    auto) auto_mode ;;
    *) command_mode "$@" ;;
esac
