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
| `AT+CNF` | Saves the current configuration to file (equivalent to `AT+CNF=save`). **Requires login.** |
| `AT+CNF?` | Returns `<serial>,<timezone>` |
| `AT+CNF=?` | Returns the syntax: `serial,<value> \| timezone,<value> \| save \| load` |
| `AT+CNF=serial,<value>` | Sets the serial number (max 16 characters). **Requires login.** |
| `AT+CNF=timezone,<value>` | Sets the timezone (`i16`, minutes) and applies it immediately. **Requires login.** |
| `AT+CNF=save` | Saves the current configuration to file. **Requires login.** |
| `AT+CNF=load` | Reloads the configuration from file and reapplies it (locale, DST, NTP, WiFi, session). **Requires login.** |

Example:
```
AT+CNF=serial,SN00123456
AT+CNF=timezone,60
AT+CNF=save
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
| `start_month` | `u8` | DST start month |
| `start_day` | `u8` | Start day (`0xFF` = last week of the month) |
| `start_hour` | `u8` | Start hour |
| `end_month` | `u8` | DST end month |
| `end_day` | `u8` | End day (`0xFF` = last week of the month) |
| `end_hour` | `u8` | End hour |
| `enabled` | `<0\|1>` | Enables/disables DST |

Every `set` immediately applies the new DST configuration (`Config::apply_daylight_saving_time`).

Example:
```
AT+DST=start_month,3
AT+DST=enabled,1
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

## AT+SPK — Sprinkler (schedules and zones)

Module: `main/src/apps/sprinkler.rs` (`Sprinkler`), with sub-structures
`main/src/apps/sprinkler/schedule.rs` (`Schedule`) and `main/src/apps/sprinkler/zone.rs` (`Zone`).

Fixed limits: maximum **4 schedules** (`MAX_SCHEDULES`) per sprinkler, maximum
**4 zones** (`ZONES_SIZE`) per schedule.

| Form | Description |
|---|---|
| `AT+SPK` | No-op (no action) |
| `AT+SPK?` | Returns the currently selected schedule and its zones, one per line (see below). **Requires login.** |
| `AT+SPK=?` | Returns the full syntax (see below) |
| `AT+SPK=select,<index>` | Selects which schedule `AT+SPK?` will print. **Requires login.** |
| `AT+SPK=schedule,insert,...` | Inserts a schedule into a specific slot. **Requires login.** |
| `AT+SPK=schedule,delete,<index>` | Removes the schedule at the given index. **Requires login.** |
| `AT+SPK=zone,<schedule_index>,insert,...` | Inserts a zone into a specific slot of the schedule. **Requires login.** |
| `AT+SPK=zone,<schedule_index>,delete,<zone_index>` | Removes the zone at the given index. **Requires login.** |
| `AT+SPK=save` | Saves the current schedules state to file. **Requires login.** |

### `select` and `query`

```
AT+SPK=select,<index>
AT+SPK?
```

- `index` (required): position in the schedules array (`0`-`3`) to select. The selection
  is kept in memory (not persisted) and defaults to `0` at startup.

`AT+SPK?` then returns the selected schedule and all of its zones, one entry per line
(`\r\n`-separated), in a compact field-only format (no `description`, to fit the
96-byte AT response buffer):

```
<schedule_index>:<minute>,<hour>,<days>,<month>,<status>
<zone_index>:<relay_number>,<watering_time>,<weight>,<status>
<zone_index>:<relay_number>,<watering_time>,<weight>,<status>
...
```

`status` is the numeric value of the `Status` enum: `0` = `UNACTIVE`, `1` = `ACTIVE`,
`2` = `RUN`.

Example:
```
AT+SPK=select,0
AT+SPK?
```
```
0:8,255,0,0,1
0:0,10,1,1
1:1,5,0,1
2:2,0,0,0
3:3,0,0,0
```

### `schedule,insert`

```
AT+SPK=schedule,insert,<index>,[minute],[hour],[days],[month],[description]
```

- `index` (required): position in the schedules array (`0`-`3`). Must be a free slot
  (status `UNACTIVE`), otherwise the error `"schedule index already occupied"` is returned.
- `minute` (optional, `u8`): execution minute + 1 (`0`/omitted = every minute).
- `hour` (optional, `u8`): execution hour + 1 (`0`/omitted = every hour).
- `days` (optional, `u8`): weekday bitmask (`0`/omitted = every day). See the `Day` table below.
- `month` (optional, `u16`): month bitmask (`0`/omitted = every month). See the `Month` table below.
- `description` (optional, string): free text, max `DISPLAY_INPUT_MAX_SIZE` characters.

Inserting resets the schedule's zones (all set to `Zone::default()`) and sets its
status to `ACTIVE`.

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

### `schedule,delete`

```
AT+SPK=schedule,delete,<index>
```

Resets the schedule at the given index to its default state (`UNACTIVE`, no zones),
making the slot available again for a future `insert`.

### `zone,insert`

```
AT+SPK=zone,<schedule_index>,insert,<index>,<watering_time>,[weight],[description]
```

- `schedule_index` (required): index of the schedule the zone belongs to.
- `index` (required): position in the schedule's zones array (`0`-`3`). Must be a free
  slot (status `UNACTIVE`), otherwise the error `"zone index already occupied"` is
  returned. This same value is also assigned to the zone's `relay_number` field.
- `watering_time` (required, `u8`): irrigation duration in minutes.
- `weight` (optional, `u8`, default `0`): weight used for execution order (lower values run first).
- `description` (optional, string, default empty): free text, max `DISPLAY_INPUT_MAX_SIZE` characters.

Inserting sets the zone's status to `ACTIVE`.

### `zone,delete`

```
AT+SPK=zone,<schedule_index>,delete,<zone_index>
```

Resets the zone at the given index to its default state (`UNACTIVE`).

### `save`

```
AT+SPK=save
```

Serializes to JSON and saves to file (`sprinkler.json`) the current state of all
schedules and zones.

### Full example

```
# Login
AT+SESS=i,admin@hhg.local,mysecretpassword
AT+SESS

# Create schedule 0: every day at 7:00, every month
AT+SPK=schedule,insert,0,,8,,,"Morning irrigation"

# Add zone 0 to schedule 0: relay 0, 10 minutes
AT+SPK=zone,0,insert,0,10,1,"Front flowerbed"

# Add zone 1 to schedule 0: relay 1, 5 minutes
AT+SPK=zone,0,insert,1,5

# Inspect schedule 0 and its zones
AT+SPK=select,0
AT+SPK?

# Save to file
AT+SPK=save

# Remove zone 1 and then the whole schedule 0
AT+SPK=zone,0,delete,1
AT+SPK=schedule,delete,0
```

---

## Command summary

| Command | Module | Description |
|---|---|---|
| `AT+CNF` | Config | General configuration (serial, timezone, save/load) |
| `AT+DST` | DaylightSavingTime | Daylight saving time |
| `AT+WIFI` | WifiConfig | WiFi configuration |
| `AT+NTP` | NtpConfig | NTP server configuration |
| `AT+SESS` | Session | Session login/logout |
| `AT+USR` | User | Local user |
| `AT+SYS` | SystemHandler | Reset/factory reset/system status |
| `AT+SPK` | Sprinkler | Irrigation schedules and zones |
