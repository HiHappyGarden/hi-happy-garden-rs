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
# Configures a schedule (0 by default) over the AT command port so that it
# fires <offset> seconds from now (60 by default). The scheduler matches on
# hour and minute only, so the target time is rounded up to the next whole
# minute: it never fires earlier than the offset.
#
# Schedule::executable is fed the RTC time with timezone and DST applied
# (AT+CNF=tz / AT+DST), so by default hour and minute are the host local
# time: the board must be set to the same timezone, or pass --tz.

set -euo pipefail

USAGE="\
Usage: $(basename "$0") [options]

Options:
  -o, --offset <s>      seconds from now to the trigger (default: 60)
  -i, --index <n>       schedule index 0-3 (default: 0)
  -z, --zone <r>,<min>  relay and watering minutes, repeatable (default: 0,1)
  -e, --email <email>   login email (default: \$HHG_EMAIL)
  -p, --password <pw>   login password (default: \$HHG_PASSWORD)
  -d, --device <tty>    AT serial port (default: /dev/ttyACM0)
  -b, --baud <rate>     baud rate (default: 115200)
      --tz <minutes>    board offset from UTC in minutes, DST included
                        (default: the host local time)
  -s, --save            persist the schedules to file (AT+SCH=<idx>,sv)
  -n, --print           print the commands instead of sending them (e.g. to paste
                        into the emulator, whose AT port is its stdin)
  -h, --help            print this help"

OFFSET=60
INDEX=0
ZONES=()
EMAIL="${HHG_EMAIL:-}"
PASSWORD="${HHG_PASSWORD:-}"
DEVICE=/dev/ttyACM0
BAUD=115200
TZ_MIN=""
SAVE=0
PRINT=0
REPLY_TIMEOUT=3

die() { echo "$(basename "$0"): $*" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        -o|--offset)   OFFSET="$2"; shift 2 ;;
        -i|--index)    INDEX="$2"; shift 2 ;;
        -z|--zone)     ZONES+=("$2"); shift 2 ;;
        -e|--email)    EMAIL="$2"; shift 2 ;;
        -p|--password) PASSWORD="$2"; shift 2 ;;
        -d|--device)   DEVICE="$2"; shift 2 ;;
        -b|--baud)     BAUD="$2"; shift 2 ;;
        --tz)          TZ_MIN="$2"; shift 2 ;;
        -s|--save)     SAVE=1; shift ;;
        -n|--print)    PRINT=1; shift ;;
        -h|--help)     echo "$USAGE"; exit 0 ;;
        *)             die "unknown option: $1 (try --help)" ;;
    esac
done

[[ "$OFFSET" =~ ^[0-9]+$ ]] || die "offset must be a number of seconds"
[[ "$INDEX" =~ ^[0-3]$ ]] || die "index must be 0-3"
[[ -z "$TZ_MIN" || "$TZ_MIN" =~ ^-?[0-9]+$ ]] || die "tz must be minutes, e.g. 60 or -300"
[[ ${#ZONES[@]} -gt 0 ]] || ZONES=("0,1")
for z in "${ZONES[@]}"; do
    [[ "$z" =~ ^[0-3],[0-9]+$ ]] || die "zone must be <relay 0-3>,<minutes>: $z"
done
if [[ $PRINT -eq 0 && ( -z "$EMAIL" || -z "$PASSWORD" ) ]]; then
    die "login needed: pass --email/--password or set HHG_EMAIL/HHG_PASSWORD"
fi

# Target = now + offset, rounded up to the next whole minute
NOW=$(date +%s)
TARGET=$(( (NOW + OFFSET + 59) / 60 * 60 ))
if [[ -n "$TZ_MIN" ]]; then
    read -r HOUR MINUTE CLOCK < <(TZ=UTC date -d "@$(( TARGET + TZ_MIN * 60 ))" '+%-H %-M %H:%M')
else
    read -r HOUR MINUTE CLOCK < <(date -d "@$TARGET" '+%-H %-M %H:%M')
fi
LOCAL_CLOCK=$(date -d "@$TARGET" '+%H:%M')

# Raw stored values are +1 (0 means "every")
COMMANDS=(
    "AT+SESS=i,${EMAIL:-<email>},${PASSWORD:-<password>}"
    "AT+SESS"
    "AT+SCH=${INDEX},hr,$(( HOUR + 1 ))"
    "AT+SCH=${INDEX},mi,$(( MINUTE + 1 ))"
    "AT+SCH=${INDEX},dy,0"
    "AT+SCH=${INDEX},mo,0"
    "AT+SCH=${INDEX},ds,\"Test ${CLOCK}\""
)
for z in "${ZONES[@]}"; do
    COMMANDS+=("AT+SCH=${INDEX},zn,${z}")
done
COMMANDS+=("AT+SCH=${INDEX},st,1" "AT+SCH?" "AT+SCH")
[[ $SAVE -eq 1 ]] && COMMANDS+=("AT+SCH=${INDEX},sv")

echo "schedule ${INDEX} -> ${CLOCK} board clock, ${LOCAL_CLOCK} local (in $(( TARGET - NOW )) s, zones: ${ZONES[*]})" >&2

if [[ $PRINT -eq 1 ]]; then
    printf '%s\n' "${COMMANDS[@]}"
    exit 0
fi

[[ -c "$DEVICE" ]] || die "no serial port $DEVICE"
if pgrep -a picocom >/dev/null || pgrep -a minicom >/dev/null; then
    echo "warning: a terminal is open on a serial port, replies may be split with it" >&2
fi

stty -F "$DEVICE" "$BAUD" raw -echo -hupcl cs8 -cstopb -parenb
exec 3<>"$DEVICE"

# Sends one command and waits for its reply line (+XXX: ... or bare KO),
# skipping the firmware log that shares the port
send() {
    local cmd="$1" line
    printf '%s\r\n' "$cmd" >&3
    while IFS= read -r -t "$REPLY_TIMEOUT" line <&3; do
        line="${line%$'\r'}"
        if [[ "$line" =~ ^\+[A-Z]+:\ (.*)$ || "$line" == "KO" ]]; then
            echo "$cmd -> $line"
            [[ "$line" == "KO" || "$line" =~ :\ KO ]] && return 1
            # AT+SCH? may print more, the last line is the one we read
            return 0
        fi
    done
    echo "$cmd -> no reply" >&2
    return 1
}

for cmd in "${COMMANDS[@]}"; do
    if [[ "$cmd" == AT+SESS=i,* ]]; then
        # Do not echo the password
        printf '%s\r\n' "$cmd" >&3
        while IFS= read -r -t "$REPLY_TIMEOUT" line <&3; do
            line="${line%$'\r'}"
            [[ "$line" =~ ^\+SESS:\ (.*)$ ]] && { echo "AT+SESS=i,${EMAIL},*** -> $line"; break; }
        done
        continue
    fi
    send "$cmd" || die "failed at: $cmd"
done

exec 3>&-
echo "done: schedule ${INDEX} fires at ${LOCAL_CLOCK} local (${CLOCK} board clock)" >&2
