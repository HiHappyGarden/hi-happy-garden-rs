# AT Commands — Hi Happy Garden

Documentation of the AT commands exposed by the firmware, one per application module
in `main/src/apps/`. The commands are registered in `main/src/apps/parser.rs`. Each
command supports up to four forms, depending on the suffix sent after the command name:

| Suffix     | Form     | Meaning                                        |
|------------|----------|-------------------------------------------------|
| *(none)*   | Exec     | Executes an action without parameters           |
| `?`        | Query    | Reads the current state/value                   |
| `=?`       | Test     | Lists the syntax of the accepted parameters      |
| `=<args>`  | Set      | Sets values/performs an operation with arguments |

Not every command implements all four forms; unsupported forms return
`AtError::NotSupported`. See each section below for which forms are available.

Arguments in the Set form are comma-separated. Values that contain commas must be
wrapped in double quotes (e.g. `"hello, world"`); to include a literal quote use `\"`.

Notation used in this document:
- `<value>` — required argument
- `[value]` — optional argument (a default is used if omitted)
- `<a|b>` — choice between alternative literal values
- `<n-m>` — allowed numeric range

## Responses

Every response line starts with the command's response prefix (e.g. `+CNF: `) and ends
with `\r\n`:

| Outcome | Response |
|---|---|
| Success with a payload | `+XXX: <payload>` |
| Success with no payload | `+XXX: OK` |
| `AtError::Unhandled("<msg>")` | `+XXX: <msg>` (e.g. `+CNF: serial max len 16`) |
| `AtError::InvalidArgs` / `NotSupported` / any other error | `+XXX: KO` |
| Unknown command | `KO` |

Commands marked as **requires login** respond with `+XXX: KO: Not logged in` (via
`AtError::Unhandled`) if there is no active user session (flag
`StatusFlag::UserLogged`).

Session and source: a session belongs to the channel (UART or MQTT) that opened it.
While a user is logged in from one source, **every** command coming from the other
source is rejected with a bare `KO`. Every command processed while logged in resets the
5-minute auto-logout timer.

---

## AT+CNF — General configuration

Module: `main/src/apps/config.rs` (`Config`)

| Form | Description |
|---|---|
| `AT+CNF` | Not supported. |
| `AT+CNF?` | Returns `"<serial>",<timezone>` |
| `AT+CNF=?` | Returns the syntax: `sn,<value> \| tz,<value> \| sv` |
| `AT+CNF=sn,<value>` | Sets the serial number (max 16 characters). **Requires login.** |
| `AT+CNF=tz,<value>` | Sets the timezone (`i16`, minutes) and applies it immediately. **Requires login.** |
| `AT+CNF=sv` | Saves the current configuration to file (`config.json`). **Requires login.** |

`AT+CNF=sv` persists the whole `Config` structure, which also embeds DST, WiFi, NTP and
the session users: changes made through `AT+DST`, `AT+WIFI`, `AT+NTP` and `AT+USR` are
applied immediately but are only persisted once `AT+CNF=sv` is issued.

Example:
```
AT+CNF=sn,SN00123456
AT+CNF=tz,60
AT+CNF=sv
AT+CNF?
```

---

## AT+DST — Daylight Saving Time

Module: `main/src/apps/config.rs` (`DaylightSavingTime`)

| Form | Description |
|---|---|
| `AT+DST` | Not supported. |
| `AT+DST?` | Returns `<start_month>,<start_day>,<start_hour>,<end_month>,<end_day>,<end_hour>,<enabled>` |
| `AT+DST=?` | Returns the syntax: `<smo\|sdy\|shr\|emo\|edy\|ehr\|en>,<value>` |
| `AT+DST=<field>,<value>` | Sets a single field. **Requires login.** |

Fields settable with `AT+DST=<field>,<value>`:

| Field | Range | Notes |
|---|---|---|
| `smo` | `1-12` | DST start month |
| `sdy` | `1-31` or `255` | Start day (`255` = `0xFF` = last week of the month) |
| `shr` | `0-23` | Start hour |
| `emo` | `1-12` | DST end month |
| `edy` | `1-31` or `255` | End day (`255` = `0xFF` = last week of the month) |
| `ehr` | `0-23` | End hour |
| `en` | `<0\|1>` | Enables/disables DST |

Out-of-range values return `KO`. Every `set` immediately applies the new DST
configuration (`Config::apply_daylight_saving_time`); persist with `AT+CNF=sv`.

Example:
```
AT+DST=smo,3
AT+DST=sdy,255
AT+DST=en,1
AT+DST?
```

---

## AT+WIFI — WiFi configuration

Module: `main/src/apps/config.rs` (`WifiConfig`)

| Form | Description |
|---|---|
| `AT+WIFI` | Not supported. |
| `AT+WIFI?` | Returns `"<ssid>",<auth>,<enabled>` (the password is never returned) |
| `AT+WIFI=?` | Returns the syntax: `<ssid>,<password>,<auth 0-6>,<enabled 0\|1>` |
| `AT+WIFI=<ssid>,<password>,<auth>,<enabled>` | Sets the WiFi configuration. **Requires login.** |

Constraints:
- `ssid` and `password`: max 32 characters each.
- `auth`: integer `0-6`, mapped to the `Auth` enum (any other value falls back to `Open`):

  | Value | Auth |
  |---|---|
  | 0 | Open |
  | 1 | Wep |
  | 2 | Wpa |
  | 3 | Wpa2 |
  | 4 | Wpa2Mixed |
  | 5 | Wpa3 |
  | 6 | Wpa2Wpa3 |
- `enabled`: `0` = disabled, any non-zero value = enabled.

The `set` immediately applies the WiFi configuration (`Config::apply_wifi`); persist with
`AT+CNF=sv`.

Example:
```
AT+WIFI="MyNetwork",supersecret,3,1
AT+WIFI?
```

---

## AT+NTP — NTP server configuration

Module: `main/src/apps/config.rs` (`NtpConfig`)

| Form | Description |
|---|---|
| `AT+NTP` | Not supported. |
| `AT+NTP?` | Returns `"<server>",<port>,<msg_len>` |
| `AT+NTP=?` | Returns the syntax: `<server>,<port>,<msg_len>` |
| `AT+NTP=<server>,<port>,<msg_len>` | Sets the NTP server. **Requires login.** |

Constraints: `server` max 64 characters; `port` and `msg_len` are `u16`.

The `set` immediately applies the NTP configuration (`Config::apply_ntp`); persist with
`AT+CNF=sv`.

Example:
```
AT+NTP=pool.ntp.org,123,48
AT+NTP?
```

---

## AT+SESS — User session (login/logout)

Module: `main/src/apps/session.rs` (`Session`)

| Form | Description |
|---|---|
| `AT+SESS` | If temporary credentials are staged, attempts the login: returns `<email>` on success, `KO` on failure. If no credentials are staged, logs out (returns `OK`), or `KO: Not logged in` if no one is logged in. |
| `AT+SESS?` | Returns the email of the currently logged-in user, or `KO` if no one is logged in. |
| `AT+SESS=?` | Returns the syntax: `<i\|o>,<email>,<password>` |
| `AT+SESS=i,<email>,<password>` | Stages the login credentials (verified later by `AT+SESS`). |
| `AT+SESS=o` | Clears the staged credentials, so that the next `AT+SESS` logs out. |

Notes:
- Login compares the credentials against the known users (system user, slot 0, and
  local user, slot 1).
- A failed login clears the staged credentials if no one is logged in.
- An active session automatically expires after 5 minutes of inactivity (auto-logout
  timer), reset on every command processed while logged in.
- Logout also clears the temporary local user (`AT+USR`) and the
  `UartCmd`/`MqttCmd`/`SystemCmd` status flags.

Example (login):
```
AT+SESS=i,admin@hhg.local,mysecretpassword
AT+SESS
AT+SESS?
```

Example (logout):
```
AT+SESS=o
AT+SESS
```

---

## AT+USR — Local user

Module: `main/src/apps/session.rs` (`User`)

| Form | Description |
|---|---|
| `AT+USR` | Copies the temporary local user into the session as the local user (slot 1), clears the temporary user and returns its `<email>`. **Requires login.** |
| `AT+USR?` | Returns `"<email>"` of the temporary local user. **Requires login.** |
| `AT+USR=?` | Returns the syntax: `<email>,<password>` |
| `AT+USR=<email>,<password>` | Sets the email and password (SHA-256 hash) of the temporary local user. **Requires login.** |

Constraints: `email` and `password` max 32 characters.

The new local user lives in RAM only until `AT+CNF=sv` is issued.

Example:
```
AT+USR=passy.linux@zresa.it,12345678
AT+USR
AT+CNF=sv
```

---

## AT+SYS — System

Module: `main/src/apps/system_handler.rs` (`SystemHandler`)

| Form | Description |
|---|---|
| `AT+SYS` | Not supported. |
| `AT+SYS?` | Returns `<hardware_error>,<error>,<status>` (current signal bitmasks) |
| `AT+SYS=?` | Returns the syntax: `<rs\|fr\|hwe\|e\|s>` |
| `AT+SYS=rs` | Resets the system (`Hardware::reset`). **Requires login.** |
| `AT+SYS=fr` | Factory reset: recursively removes the filesystem, unmounts it, and reboots. **Requires login.** |
| `AT+SYS=hwe` | Returns the hardware error signal. **Requires login.** |
| `AT+SYS=e` | Returns the error signal. **Requires login.** |
| `AT+SYS=s` | Returns the status signal. **Requires login.** |

Verb legend: `rs` = reset/reboot, `fr` = factory reset, `hwe` = hardware error, `e` = error, `s` = status.

`AT+SYS?` does not require login; the single-signal `hwe`/`e`/`s` forms do.

> **Warning:** `AT+SYS=fr` is destructive and erases all persistent data (configuration, sessions, schedules, zones).

Example:
```
AT+SYS?
AT+SYS=s
AT+SYS=rs
```

---

## AT+SCH — Irrigation schedules

Module: `main/src/apps/sprinkler/schedule.rs` (`ScheduleController`, `Schedule`)

Fixed limit: **4 schedules** (`ScheduleController::SIZE`), indexes `0-3`. There is no
insert/delete: the 4 schedules always exist and are edited in place.

| Form | Description |
|---|---|
| `AT+SCH` | Commits the staged schedule to the live schedule at the staged index. Fails with `No modify applied` if nothing is staged. **Requires login.** |
| `AT+SCH?` | Returns the **staged** schedule (see format below). |
| `AT+SCH=?` | Returns the syntax: `<idx>,<mi\|hr\|dy\|mo\|ds\|zn\|st>,<value> \| sv` |
| `AT+SCH=<idx>,<field>,<value>` | Stages a change to schedule `idx`. **Requires login.** |
| `AT+SCH=<idx>,zn,<zone_relay>,<minutes>` | Stages a zone assignment for schedule `idx` (see below). **Requires login.** |
| `AT+SCH=<idx>,sv` | Persists **all** live schedules to file (`schedules.json`). **Requires login.** |

### Stage-then-commit flow

`AT+SCH` follows the same flow as [`AT+ZN`](#atzn--irrigation-zones-relays):

1. The first `AT+SCH=<idx>,...` copies live schedule `idx` into a staging buffer
   (`SCHEDULE_TMP`) and applies the field change to the buffer.
2. Further `set`s on the **same** `idx` keep accumulating in the buffer. A `set` on a
   **different** `idx` reloads the buffer from that schedule, discarding the pending
   changes.
3. The bare `AT+SCH` (Exec) copies the buffer into the live schedule and clears the buffer.
4. `AT+SCH=<idx>,sv` serializes the live `ScheduleController` to file. Issue `AT+SCH`
   **before** `sv`, otherwise staged changes are not saved. `idx` is required (any valid
   index) and, like every `set`, reloads the buffer if it differs from the staged index.

### `AT+SCH?` response format

```
<idx>,<minute>,<hour>,<days>,<month>,<description>,[<zone>, <zone>, <zone>, <zone>]
```

Each `<zone>` is either `Relay N=<minutes>` or `None`. The values are the raw stored
ones (minute/hour are `+1`, see below); the description is not quoted. After a commit
(or at boot) the buffer is empty and the response shows index `0` with a blank schedule.

### Fields

Fields settable with `AT+SCH=<idx>,<field>,<value>`:

| Field | Range | Notes |
|---|---|---|
| `mi` | `0-60` | Minute + 1 (`0` = every minute) |
| `hr` | `0-24` | Hour + 1 (`0` = every hour) |
| `dy` | `0-127` (`0x7F`) | Weekday bitmask (`0` = every day). See the `Day` table below. |
| `mo` | `0-4095` (`0x0FFF`) | Month bitmask (`0` = every month). See the `Month` table below. |
| `ds` | string | Description, max `DISPLAY_INPUT_MAX_SIZE` characters. |
| `zn` | `<zone_relay>,<minutes>` | Assigns/updates a zone on this schedule (two values, see below). |
| `st` | `<0\|1>` | Status: `0` = `UNACTIVE`, `1` = `ACTIVE`. `2` (`RUN`) is set only by the firmware and is rejected. |

Values are decimal integers; out-of-range values return `KO`.

`Day` bitmask:

| Bit | Day |
|---|---|
| `0x01` | Sunday |
| `0x02` | Monday |
| `0x04` | Tuesday |
| `0x08` | Wednesday |
| `0x10` | Thursday |
| `0x20` | Friday |
| `0x40` | Saturday |

`Month` bitmask:

| Bit | Month | Bit | Month |
|---|---|---|---|
| `0x0001` | January | `0x0080` | August |
| `0x0002` | February | `0x0100` | September |
| `0x0004` | March | `0x0200` | October |
| `0x0008` | April | `0x0400` | November |
| `0x0010` | May | `0x0800` | December |
| `0x0020` | June | | |
| `0x0040` | July | | |

### `zn` — assigning a zone to a schedule

```
AT+SCH=<idx>,zn,<zone_relay>,<minutes>
```

- `zone_relay` (required, `0-3`): which physical relay (see [`AT+ZN`](#atzn--irrigation-zones-relays)) to run as part of this schedule.
- `minutes` (required, `u8`): watering time in minutes for that relay within this
  schedule; `0` effectively disables the zone for this schedule.

The zone is looked up by `zone_relay` among the schedule's already-assigned zones; if
found its minutes are updated in place, otherwise it is written into the first free slot
(up to 4 zones per schedule). Zones cannot be removed from a schedule once assigned; set
their minutes to `0` instead.

### Example

```
# Login
AT+SESS=i,admin@hhg.local,mysecretpassword
AT+SESS

# Stage schedule 0: every day at 7:00 (hour+1=8, minute+1=1), with a description
AT+SCH=0,hr,8
AT+SCH=0,mi,1
AT+SCH=0,ds,"Morning irrigation"

# Assign relay 0 for 10 minutes and relay 1 for 5 minutes
AT+SCH=0,zn,0,10
AT+SCH=0,zn,1,5

# Activate it
AT+SCH=0,st,1

# Inspect the staged schedule, then commit it
AT+SCH?
AT+SCH

# Save all schedules to file
AT+SCH=0,sv
```

---

## AT+ZN — Irrigation zones (relays)

Module: `main/src/apps/sprinkler/zone.rs` (`ZoneController`, `Zone`)

Fixed limit: 4 zones (`ZoneController::SIZE`), one per physical relay (`Relay0`-`Relay3`).
Zones are a fixed pool tied 1:1 to relays: there is no insert/delete, only editing a
zone's `weight`/`description` and persisting.

| Form | Description |
|---|---|
| `AT+ZN` | Commits the staged zone to the live zone. Fails with `No modify applied` if nothing is staged. **Requires login.** |
| `AT+ZN?` | Returns all 4 live zones, one per line (see below). **Requires login.** |
| `AT+ZN=?` | Returns the syntax: `<zone_relay>,<wt\|ds>,<value> \| sv` |
| `AT+ZN=<zone_relay>,wt,<value>` | Stages a new weight (`u8`) for the zone. **Requires login.** |
| `AT+ZN=<zone_relay>,ds,<value>` | Stages a new description (max `DISPLAY_INPUT_MAX_SIZE` characters). **Requires login.** |
| `AT+ZN=<zone_relay>,sv` | Saves all live zones to file (`zones.json`). **Requires login.** |

`zone_relay` is `0-3`.

`AT+ZN` follows a stage-then-commit flow: `set` copies the target zone into a temporary
buffer (`ZONE_TMP`) and applies the requested field change to it; further `set`s on the
same relay accumulate, while a `set` on a different relay reloads the buffer and
discards the pending changes. The bare `AT+ZN` (Exec) then copies `weight`/`description`
from the buffer into the live zone. Nothing is persisted to file until
`AT+ZN=<zone_relay>,sv` is issued afterwards.

`AT+ZN?` response format, one line per zone (`\r\n`-separated):

```
<zone_relay>,<weight>,"<description>"
```

Example:
```
# Login
AT+SESS=i,admin@hhg.local,mysecretpassword
AT+SESS

# Stage and commit a new description + weight for relay 0
AT+ZN=0,ds,"Front flowerbed"
AT+ZN=0,wt,1
AT+ZN

# Inspect all zones
AT+ZN?

# Persist to file
AT+ZN=0,sv
```

---

## Command summary

| Command | Module | Exec | Query | Test | Set |
|---|---|---|---|---|---|
| `AT+CNF` | Config | – | ✓ | ✓ | `sn`, `tz`, `sv` 🔒 |
| `AT+DST` | DaylightSavingTime | – | ✓ | ✓ | `smo`…`en` 🔒 |
| `AT+WIFI` | WifiConfig | – | ✓ | ✓ | ✓ 🔒 |
| `AT+NTP` | NtpConfig | – | ✓ | ✓ | ✓ 🔒 |
| `AT+SESS` | Session | login/logout | ✓ | ✓ | `i`, `o` |
| `AT+USR` | User | ✓ 🔒 | ✓ 🔒 | ✓ | ✓ 🔒 |
| `AT+SYS` | SystemHandler | – | ✓ | ✓ | `rs`, `fr`, `hwe`, `e`, `s` 🔒 |
| `AT+SCH` | ScheduleController | commit 🔒 | staged | ✓ | `mi`…`st`, `zn`, `sv` 🔒 |
| `AT+ZN` | ZoneController | commit 🔒 | ✓ 🔒 | ✓ | `wt`, `ds`, `sv` 🔒 |

🔒 = requires login.
