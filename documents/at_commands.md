# AT Commands — Hi Happy Garden

Documentation of the AT commands exposed by the firmware, one per application module
in `main/src/apps/`. Each command supports up to four forms, depending on the suffix
sent after the command name:

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

Commands marked as **requires login** respond with `KO: Not logged in` (via
`AtError::Unhandled`) if there is no active user session (flag
`StatusFlag::UserLogged`).

---

## AT+CNF — General configuration

Module: `main/src/apps/config.rs` (`Config`)

| Form | Description |
|---|---|
| `AT+CNF?` | Returns `<serial>,<timezone>` |
| `AT+CNF=?` | Returns the syntax: `sn,<value> \| tz,<value> \| sv` |
| `AT+CNF=sn,<value>` | Sets the serial number (max 16 characters). **Requires login.** |
| `AT+CNF=tz,<value>` | Sets the timezone (`i16`, minutes) and applies it immediately. **Requires login.** |
| `AT+CNF=sv` | Saves the current configuration to file. **Requires login.** |

`AT+CNF` has no bare Exec form, and `load` has been removed (no more `AT+CNF=ld`).

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
| `AT+DST?` | Returns `<start_month>,<start_day>,<start_hour>,<end_month>,<end_day>,<end_hour>,<enabled>` |
| `AT+DST=?` | Returns the syntax of the settable fields |
| `AT+DST=<field>,<value>` | Sets a single field. **Requires login.** |

Fields settable with `AT+DST=<field>,<value>`:

| Field | Type | Notes |
|---|---|---|
| `smo` | `u8` | DST start month |
| `sdy` | `u8` | Start day (`0xFF` = last week of the month) |
| `shr` | `u8` | Start hour |
| `emo` | `u8` | DST end month |
| `edy` | `u8` | End day (`0xFF` = last week of the month) |
| `ehr` | `u8` | End hour |
| `en` | `<0\|1>` | Enables/disables DST |

Every `set` immediately applies the new DST configuration (`Config::apply_daylight_saving_time`).

Example:
```
AT+DST=smo,3
AT+DST=en,1
AT+DST?
```

---

## AT+WIFI — WiFi configuration

Module: `main/src/apps/config.rs` (`WifiConfig`)

| Form | Description |
|---|---|
| `AT+WIFI?` | Returns `"<ssid>",<auth>,<enabled>` |
| `AT+WIFI=?` | Returns the syntax: `<ssid>,<password>,<auth 0-6>,<enabled 0\|1>` |
| `AT+WIFI=<ssid>,<password>,<auth>,<enabled>` | Sets the WiFi configuration. **Requires login.** |

Constraints:
- `ssid` and `password`: max 32 characters each.
- `auth`: integer `0-6`, mapped to the `Auth` enum:

  | Value | Auth |
  |---|---|
  | 0 | Open |
  | 1 | Wep |
  | 2 | Wpa |
  | 3 | Wpa2 |
  | 4 | Wpa2Mixed |
  | 5 | Wpa3 |
  | 6 | Wpa2Wpa3 |
- `enabled`: `0` or `1`.

The `set` immediately applies the WiFi configuration (`Config::apply_wifi`).

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
| `AT+NTP?` | Returns `"<server>",<port>,<msg_len>` |
| `AT+NTP=?` | Returns the syntax: `<server>,<port>,<msg_len>` |
| `AT+NTP=<server>,<port>,<msg_len>` | Sets the NTP server. **Requires login.** |

Constraints: `server` max 64 characters; `port` and `msg_len` are `u16`.

The `set` immediately applies the NTP configuration (`Config::apply_ntp`).

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
| `AT+SESS` | Logs in if valid temporary credentials are present, otherwise logs out (if a user is logged in). |
| `AT+SESS?` | Returns the email of the currently logged-in user, or an error if no one is logged in. |
| `AT+SESS=?` | Returns the syntax: `<i\|o>,<email>,<password>` |
| `AT+SESS=i,<email>,<password>` | Prepares the login credentials (verified later by `AT+SESS`). |
| `AT+SESS=o` | Prepares the logout (executed later by `AT+SESS`). |

Notes:
- Login compares the credentials against the known users (system user and local user).
- An active session automatically expires after 5 minutes of inactivity (auto-logout timer), reset on every authenticated AT command.
- Logout also clears the `UartCmd`/`MqttCmd`/`SystemCmd` status flags.

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
| `AT+USR` | Confirms the temporary local user as the definitive local user (slot 1 of `Session`). **Requires an active session.** |
| `AT+USR?` | Returns `"<email>"` of the temporary local user. **Requires an active session.** |
| `AT+USR=?` | Returns the syntax: `<email>,<password>` |
| `AT+USR=<email>,<password>` | Sets the email and password (SHA-256 hash) of the temporary local user. **Requires an active session.** |

Constraints: `email` and `password` max 32 characters.

Example:
```
AT+USR=passy.linux@zresa.it,12345678
AT+USR
AT+USR?
```

---

## AT+SYS — System

Module: `main/src/apps/system_handler.rs` (`SystemHandler`)

| Form | Description |
|---|---|
| `AT+SYS?` | Returns `<hardware_error>,<error>,<status>` (current signals) |
| `AT+SYS=?` | Returns the syntax: `<rs\|fr\|hwe\|e\|s>` |
| `AT+SYS=rs` | Resets the system (`Hardware::reset`). **Requires login.** |
| `AT+SYS=fr` | Factory reset: recursively removes the filesystem, unmounts it, and reboots. **Requires login.** |

Verb legend: `rs` = reset/reboot, `fr` = factory reset, `hwe` = hardware error, `e` = error, `s` = status.

> **Warning:** `AT+SYS=fr` is destructive and erases all persistent data (configuration, sessions, irrigation schedules).

Example:
```
AT+SYS=?
AT+SYS=rs
```

---

## AT+SCH — Irrigation schedules

Module: `main/src/apps/sprinkler/schedule.rs` (`ScheduleController`, `Schedule`)

Fixed limit: maximum **4 schedules** (`ScheduleController::SIZE`).

`AT+SPK` no longer exists as an AT command: schedules and zones are now handled by two
independent commands, `AT+SCH` (this section) and [`AT+ZN`](#atzn--irrigation-zones-relays).
There is no more `select`/per-schedule `query`, `insert`/`delete`, or combined
schedule+zone response — see notes below for what replaces each of these.

| Form | Description |
|---|---|
| `AT+SCH` | Not supported. |
| `AT+SCH?` | Not supported. |
| `AT+SCH=?` | Returns the syntax: `<idx>,<mi\|hr\|dy\|mo\|ds\|zn\|st>,<value> \| sv` |
| `AT+SCH=<idx>,<field>,<value>` | Stages a change to schedule `idx` in a temporary buffer. **Requires login.** |
| `AT+SCH=<idx>,zn,<zone_relay>,<minutes>` | Stages a zone assignment for schedule `idx` (see below). **Requires login.** |
| `AT+SCH=<idx>,sv` | Persists **all** schedules (the whole `ScheduleController`, not just `idx`) to file. **Requires login.** |

Notes:
- Every `set` first stores `idx` as the "current" schedule index and edits a module-level
  staging buffer (`SCHEDULE_TMP`), not the live schedule directly — there is no separate
  `exec` step that commits it (unlike `AT+ZN`, see below); the field setters below write
  straight into the staging buffer, which is what `sv` serializes together with the rest
  of `SHARED`.
- There is no `query` (`AT+SCH?`) any more: schedule state cannot currently be read back
  over AT commands.
- There is no `schedule,insert` / `schedule,delete` pair any more, and no explicit way to
  remove a zone from a schedule — zones are upserted only (see `zn` below).

Fields settable with `AT+SCH=<idx>,<field>,<value>`:

| Field | Type | Notes |
|---|---|---|
| `mi` | `u8` | Minute + 1 (`0` = every minute) |
| `hr` | `u8` | Hour + 1 (`0` = every hour) |
| `dy` | `u8` | Weekday bitmask (`0` = every day). See the `Day` table below. |
| `mo` | `u16` | Month bitmask (`0` = every month). See the `Month` table below. |
| `ds` | string | Description, max `DISPLAY_INPUT_MAX_SIZE` characters. |
| `zn` | `<zone_relay>,<minutes>` | Assigns/updates a zone on this schedule (two values, see below). |
| `st` | `<0\|1\|2>` | Status: `0` = `UNACTIVE`, `1` = `ACTIVE`, `2` = `RUN`. |

`Day` bitmask (`main/src/apps/sprinkler/schedule.rs`):

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

- `zone_relay` (required, `0`-`3`): which physical relay (see [`AT+ZN`](#atzn--irrigation-zones-relays)) to run as part of this schedule.
- `minutes` (required, `u8`): watering time in minutes for that relay within this schedule.

The zone is looked up by `zone_relay` among the schedule's already-assigned zones; if
found its minutes are updated in place, otherwise it is written into the first free slot
(up to `ZoneController::SIZE`, i.e. 4 zones per schedule). There is no command to remove
a zone from a schedule once assigned.

### `sv`

```
AT+SCH=<idx>,sv
```

Serializes to JSON and saves to file (`schedules.json`) the current state of **all**
schedules (the live `ScheduleController`, `SHARED`) — note this does not include whatever
is currently staged in `SCHEDULE_TMP` for the field setters above, since those only take
effect once written back to the live schedule.

### Example

```
# Login
AT+SESS=i,admin@hhg.local,mysecretpassword
AT+SESS

# Configure schedule 0: every day at 7:00 (hour+1=8), every month, with a description
AT+SCH=0,hr,8
AT+SCH=0,ds,"Morning irrigation"

# Assign zone (relay) 0 to schedule 0 for 10 minutes, and relay 1 for 5 minutes
AT+SCH=0,zn,0,10
AT+SCH=0,zn,1,5

# Activate the schedule
AT+SCH=0,st,1

# Save all schedules to file
AT+SCH=0,sv
```

---

## AT+ZN — Irrigation zones (relays)

Module: `main/src/apps/sprinkler/zone.rs` (`ZoneController`, `Zone`)

Fixed limit: 4 zones (`ZoneController::SIZE`), one per physical relay (`Relay0`-`Relay3`).
Unlike schedules, zones are a fixed pool tied 1:1 to relays — there is no insert/delete,
only editing a zone's `weight`/`description` and persisting.

| Form | Description |
|---|---|
| `AT+ZN` | Commits the change staged by the last `AT+ZN=<zone_relay>,<wt\|ds>,<value>` to the live zone. Fails with `"No modify applied"` if nothing was staged. **Requires login.** |
| `AT+ZN?` | Returns all 4 zones, one per line (see below). **Requires login.** |
| `AT+ZN=?` | Returns the syntax: `<zone_relay>,<wt\|ds>,<value> \| sv` |
| `AT+ZN=<zone_relay>,wt,<value>` | Stages a new weight for the zone. **Requires login.** |
| `AT+ZN=<zone_relay>,ds,<value>` | Stages a new description for the zone. **Requires login.** |
| `AT+ZN=<zone_relay>,sv` | Saves the current state of all zones to file. **Requires login.** |

`AT+ZN` follows a stage-then-commit flow: `set` copies the target zone into a temporary
buffer (`ZONE_TMP`) and applies the requested field change to it; the bare `AT+ZN` (Exec)
then copies `weight`/`description` from the buffer into the real zone. Nothing is
persisted to file until `AT+ZN=<zone_relay>,sv` is issued afterwards.

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

| Command | Module | Description |
|---|---|---|
| `AT+CNF` | Config | General configuration (serial, timezone, save) |
| `AT+DST` | DaylightSavingTime | Daylight saving time |
| `AT+WIFI` | WifiConfig | WiFi configuration |
| `AT+NTP` | NtpConfig | NTP server configuration |
| `AT+SESS` | Session | Session login/logout |
| `AT+USR` | User | Local user |
| `AT+SYS` | SystemHandler | Reset/factory reset/system status |
| `AT+SCH` | ScheduleController | Irrigation schedules (fields, zone assignment, save) |
| `AT+ZN` | ZoneController | Irrigation zones/relays (weight, description, save) |
