# Bug aperti — Hi Happy Garden firmware

Stato al 2026-10-03. I bug sono emersi durante la scrittura della suite di test on-target
(`-DHHG_TESTS=ON`). Dove indicato, la suite contiene un test che oggi **fallisce** e che
passerà quando il bug sarà corretto. I bug marcati *senza test* farebbero andare in panic
la suite (con `panic = "abort"` si fermerebbe tutto), quindi non sono coperti da un test.

> **Esecuzione su HW del 2026-10-03 (Pico 2 W):** 137 test passati, 11 falliti. Gli 11
> fallimenti corrispondono ai bug 1–7, 11, 12 e 17 di questo documento: nessun altro
> test fallisce. La suite osal-rs passa per intero.
>
> **Aggiornamento 2026-10-03:** i bug 1, 2 e 3 sono corretti nel codice (il firmware compila)
> e spostati in [Risolti](#risolti). I relativi test non sono ancora stati rieseguiti su HW.

## Riepilogo

| #  | Gravità | Area | Titolo | Test |
|----|---------|------|--------|------|
| 4  | Media   | Sprinkler | `AT+ZN`: le modifiche in staging si perdono | `ZoneTests::test_set_many_then_exec` |
| 5  | Media   | Sprinkler | `AT+ZN` accetta un relè inesistente e modifica la zona 0 | `ZoneTests::test_set_unknown_relay` |
| 6  | Media   | Sprinkler | `AT+SCH` accetta valori fuori range | `ScheduleTests::test_set_out_of_range` |
| 7  | Media   | DateTime | Fine ora legale un'ora in ritardo | `DateTimeTests::test_dst_eu_fall_back` |
| 8  | Media   | Filesystem | `file_read` va in panic su errore littlefs | *senza test* |
| 9  | Media   | Parser | Panic con più righe in un solo `on_receive` | *senza test* |
| 10 | Bassa   | Drivers | `HardwareErrorFlag::from` va in panic su valore sconosciuto | *senza test* |
| 11 | Bassa   | Signals | `ErrorFlag::from(0x08)` non restituisce `DisplayHeader` | `SignalsTests::test_error_flags` |
| 12 | Bassa   | DateTime | `wday` errato per timestamp negativi | `DateTimeTests::test_negative_timestamp` |
| 13 | Bassa   | Filesystem | Errore di scrittura littlefs ignorato | *senza test* |
| 14 | Bassa   | Utils | Fallback di `deserialize_file` legge un file aperto in sola scrittura | *senza test* |
| 15 | Bassa   | Config | `AT+DST` non valida i range | *senza test* |
| 16 | Bassa   | Build | Documentazione di `HHG_DEFAULT_WIFI_AUTH` non coerente con `Auth` | *senza test* |
| 17 | Media   | Sprinkler | `AT+ZN?` restituisce solo l'ultima zona | `ZoneTests::test_query` |

---

## 4. `AT+ZN`: le modifiche in staging si perdono

**File:** [main/src/apps/sprinkler/zone.rs:290](../main/src/apps/sprinkler/zone.rs#L290)

Ogni `set` esegue `ZONE_TMP = *zone`, ricopiando la zona salvata sopra l'area di staging.

**Riproduzione:**
```
AT+ZN=0,wt,5
AT+ZN=0,ds,Orto
AT+ZN
```
Viene applicata la descrizione `Orto`, ma il peso 5 è perso.

`AT+SCH` invece accumula correttamente in `SCHEDULE_TMP`: i due comandi si comportano in modo diverso.

**Correzione proposta:** copiare la zona in `ZONE_TMP` solo se lo staging è vuoto o
riguarda un altro relè.

## 5. `AT+ZN` accetta un relè inesistente

**File:** [main/src/apps/sprinkler/zone.rs:281](../main/src/apps/sprinkler/zone.rs#L281)
(stesso schema in [schedule.rs:432](../main/src/apps/sprinkler/schedule.rs#L432) per `zn`)

`ZoneRelay::from(u8)` mappa ogni valore sconosciuto su `Relay0`, quindi `AT+ZN=7,ds,X`
modifica la zona 0 invece di rispondere errore.

**Correzione proposta:** sostituire `From<u8>` con `TryFrom<u8>` (o un controllo `< 4`) nei
punti in cui il valore arriva dall'esterno (AT, JSON).

## 6. `AT+SCH` accetta valori fuori range

**File:** [main/src/apps/sprinkler/schedule.rs:390](../main/src/apps/sprinkler/schedule.rs#L390)

`set` controlla solo che il valore stia nel tipo (`u8`/`u16`). Vengono quindi accettati
`mi,61`, `hr,25`, `dy,128`, `mo,4096`, `st,9` e il relè 7 in `zn`. Sono valori che lo
scheduler non può mai far scattare, quindi il programma resta muto senza alcun errore.
Inoltre `exec` con un indice fuori range lascia `SCHEDULE_TMP` sporco e fallisce anche ai
tentativi successivi.

**Correzione proposta:** validare i range (`mi` 0..=60, `hr` 0..=24, `dy` ≤ 0x7F,
`mo` ≤ 0x0FFF, relè < 4, `st` ≤ 2) e l'indice già in `set`; azzerare `SCHEDULE_TMP` anche quando `exec` fallisce.

## 7. Fine ora legale un'ora in ritardo

**File:** [main/src/drivers/date_time.rs:262](../main/src/drivers/date_time.rs#L262)

`is_daylight_saving_time` confronta `(mese, giorno, ora)` dell'ora **solare** locale con
`end_hour`. Il default `HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR=3` è però espresso in ora
**legale** (regola UE: 03:00 CEST = 02:00 CET = 01:00 UTC).

**Effetto:** il 27/10/2024 alle 01:30 UTC il firmware mostra 03:30 invece di 02:30. Per
un'ora all'anno l'ora locale è sbagliata e i programmi in quella fascia scattano in ritardo.

**Correzione proposta:** confrontare la fine con `end_hour - 1`, oppure documentare che
`end_hour` è in ora solare e portare il default a 2. L'inizio (`start_hour=2`, in ora solare) è già corretto.

**Output su HW (2026-10-03):**
```
[DateTimeTests] test_dst_eu_fall_back FAILED: Unhandled error owned: src/drivers/date_time/tests.rs:260: DST still active after switch: 2024-10-27 03:30:00 (UTC+01:00) DST
```

## 8. `file_read` va in panic su errore littlefs

**File:** [main/src/drivers/pico/flash.rs:129](../main/src/drivers/pico/flash.rs#L129)

Il valore restituito da `hhg_flash_read` non viene controllato: se è negativo
(ad es. `LFS_ERR_BADF` su un file aperto in sola scrittura), `len as usize` diventa
enorme e `buffer[..len]` va in panic.

**Correzione proposta:** `if len < 0 { return Err(Error::ReturnWithCode(len)) }`.

## 9. Panic con più righe in un solo `on_receive`

**File:** [main/src/apps/parser.rs:94](../main/src/apps/parser.rs#L94), righe 158 e 247

`on_receive` imposta `SOURCE` solo **dopo** aver accodato tutti i byte; il task del parser
lo azzera (`SOURCE = None`) dopo ogni riga. Se un buffer contiene due righe, alla seconda
`access_static_option!(SOURCE)` va in panic.

Oggi la UART passa un byte per volta dalla ISR, quindi il problema non si manifesta. Si
manifesterà con MQTT (`Source::Mqtt`), che passa il payload intero; in più `MQTT_CHANNEL`
non viene mai impostato.

**Correzione proposta:** accodare la sorgente insieme ai byte (oppure impostare `SOURCE`
prima di accodare) e non azzerarla tra una riga e l'altra.

## 10. `HardwareErrorFlag::from` va in panic su valore sconosciuto

**File:** [main/src/drivers/error.rs:66](../main/src/drivers/error.rs#L66)

`From<u32>` chiama `panic!` per qualunque valore non mappato, anche per combinazioni di bit
come quelle restituite da `HardwareErrorSignal::get()`.

**Correzione proposta:** usare `TryFrom<u32>`, oppure una variante `None` come per `ErrorFlag`/`StatusFlag`.

## 11. `ErrorFlag::from(0x08)` non restituisce `DisplayHeader`

**File:** [main/src/apps/signals/error.rs:40](../main/src/apps/signals/error.rs#L40)

Nel `match` manca il caso `0x08 => DisplayHeader`, quindi la conversione non fa il roundtrip.

## 12. `wday` errato per timestamp negativi

**File:** [main/src/drivers/date_time.rs:325](../main/src/drivers/date_time.rs#L325)

`timestamp / SECONDS_PER_DAY` tronca verso zero: per `-1` (1969-12-31, mercoledì) restituisce
giovedì. Su questo dispositivo è irrilevante (l'RTC parte dal 2020), ma `DateTime` dichiara
di supportare date precedenti al 1970.

**Correzione proposta:** `timestamp.div_euclid(Self::SECONDS_PER_DAY)`.

**Output su HW (2026-10-03):**
```
[DateTimeTests] test_negative_timestamp FAILED: Unhandled error owned:
```
Il messaggio dopo `Unhandled error owned:` è vuoto, anche se `test_assert_eq!` produce sempre
`file:riga: ...`. Quindi l'assert che fallisce (wday atteso a riga 129, oppure le righe 126/127)
non è identificato con certezza. Il testo potrebbe essersi perso sulla seriale. Va verificato
alla prossima esecuzione, prima di correggere.

## 13. Errore di scrittura littlefs ignorato

**File:** [main/src/drivers/pico/flash.rs:96](../main/src/drivers/pico/flash.rs#L96) e
[filesystem.rs:272](../main/src/drivers/filesystem.rs#L272)

`file_write` restituisce `Ok` anche quando `hhg_flash_write` è negativo (flash piena, I/O);
`File::write` salva poi `ret as u32` in `size`. Un salvataggio di configurazione fallito
viene quindi segnalato come riuscito.

**Correzione proposta:** propagare `Err(Error::ReturnWithCode(written))` se `written < 0`.

## 14. Fallback di `deserialize_file` legge un file aperto in sola scrittura

**File:** [main/src/apps/utils.rs:56](../main/src/apps/utils.rs#L56)

Su `-2` (NOENT) il file viene riaperto con `WRONLY | CREAT` e poi letto: la lettura fallisce
(con il bug #8 va in panic). Il ramo è raro, perché `RDWR | CREAT` non dovrebbe mai dare NOENT,
ma è sbagliato.

**Correzione proposta:** nel fallback restituire direttamente i default e salvarli, senza leggere.

## 15. `AT+DST` non valida i range

**File:** [main/src/apps/config.rs:154](../main/src/apps/config.rs#L154)

Vengono accettati `smo,13`, `shr,30` e simili. Con un mese fuori range,
`find_last_weekday_of_month` lavora su un mese di 0 giorni e il calcolo dell'ora legale diventa imprevedibile.

## 16. Documentazione di `HHG_DEFAULT_WIFI_AUTH` non coerente

**File:** [CMakeLists.txt:187](../CMakeLists.txt#L187)

La descrizione dice `0=Open,1=WPA,2=WPA2,3=WPA2-Mixed`, ma `drivers::wifi::Auth` è
`0=Open,1=Web,2=Wpa,3=Wpa2,4=Wpa2Mixed,5=Wpa3,6=Wpa2Wpa3`. Con il default `3` si ottiene
WPA2, non WPA2-Mixed come dice la documentazione.

## 17. `AT+ZN?` restituisce solo l'ultima zona

**File:** [main/src/apps/sprinkler/zone.rs:261](../main/src/apps/sprinkler/zone.rs#L261)

La query chiama `response.format(...)` per ogni zona dentro il ciclo, ma `Bytes::format`
svuota il buffer prima di scrivere: della risposta resta solo l'ultima riga. Trovato
eseguendo la suite su HW (`"0,0,\"\"\r\n"` invece di 4 righe).

**Correzione proposta:** scrivere le righe in coda (`write!(response, ...)` tramite
`core::fmt::Write`, oppure formattare ogni riga in un buffer temporaneo e usare
`append_as_sync_str`). Attenzione alla dimensione: 4 righe con descrizioni di
`DISPLAY_INPUT_MAX_SIZE` caratteri possono superare `Parser::CMD_SIZE` (96 byte).

---

## Risolti

- **#1 Maschera dei mesi spostata di uno** (Sprinkler, alta). `From<Month> for u8` ora
  restituisce 1..12, come `DateTime::month`; `Month::map` usa il bit `idx - 1`.
  Test: `SprinklerTests::test_months`, *da rieseguire su HW*.
- **#2 Programmi `UNACTIVE` eseguibili** (Sprinkler, alta). `executable()` ora restituisce
  `false` se lo stato non è `ACTIVE`. Test: `SprinklerTests::test_inactive_not_executable`,
  *da rieseguire su HW*. Resta aperto un aspetto collegato: `Sprinkler::check` mette il
  programma in `RUN` e nessuno lo riporta ad `ACTIVE` né azzera `DISBURSEMENT_IN_PROGRESS`.
  Va gestito quando si implementa l'erogazione.
- **#3 Zone di default tutte su `Relay0`, senza descrizione** (Sprinkler, alta).
  `ZoneController` e `ScheduleController` hanno ora un `Default` scritto a mano: zone
  `Relay0..Relay3` con descrizione `"Relay N"` e peso = indice, schedule con descrizione
  `"Schedule N"`. La preparazione di `SHARED` in `init` è stata rimossa. Test:
  `ZoneTests::test_zones_cover_all_relays`, `test_zones_have_description`, *da rieseguire su HW*.
  Sulle schede già inizializzate bisogna cancellare `zones.json`/`schedules.json`.
  **Attenzione:** al momento in `ZoneController::init` la chiamata a `deserialize_file` è
  commentata e in `Hardware::init_fs` è attivo `remove_recursive("/")`: entrambe le modifiche
  vanno ripristinate prima del commit.
- **osal-rs: `TaskStatus` non allineata a `TaskStatus_t`.** La struttura Rust era di
  36 byte, quella C di 48 (`pxTopOfStack`/`pxEndOfStack` con
  `configRECORD_STACK_HIGH_ADDRESS=1`, `uxCoreAffinityMask` in SMP). Di conseguenza
  `System::get_all_thread()` scriveva oltre il buffer nell'heap, `vTaskGetInfo` oltre la
  variabile sullo stack, e i campi dal secondo task in poi venivano letti sfasati.
  Ora `osal-rs-build` legge `FreeRTOSConfig.h` ed emette i cfg
  `freertos_record_stack_high_address` / `freertos_core_affinity`.
- **Build: `FREERTOS_CONFIG_PATH` non arrivava a cargo.** `CARGO_BUILD_FLAGS` era definito
  ma non usato nel comando `cargo build`, quindi osal-rs cercava `inc/FreeRTOSConfig.h`
  (inesistente). Ora la variabile è passata nell'ambiente del comando e
  `FreeRTOSConfig.h` è tra le dipendenze del target Rust.
- **osal-rs-tests: `test_system_thread_metadata`.** Pretendeva priorità > 0 per tutti i
  task, ma i task idle (`IDLE0`/`IDLE1`) girano a `tskIDLE_PRIORITY = 0`.

---

## Note non classificate come bug

- `RTC::is_to_synch()` ([rtc.rs:141](../main/src/drivers/rtc.rs#L141)) restituisce `true` quando il
  DS3231 ha un'ora **valida**: il nome suggerisce il contrario. Il comportamento è corretto
  (sincronizza POWMAN dal DS3231), ma il nome trae in inganno.
- `AppMain::check_config` ([main.rs:132](../main/src/apps/main.rs#L132)) lascia la FSM ferma in
  `CheckConfig` finché il seriale è vuoto. Probabilmente è voluto (lo imposta il wizard), da confermare.
- `Parser::commands` crea più `&mut` sullo stesso `Config::shared()` (aliasing già noto, refactor in corso).
- **Core 1 fermo nella bootrom (non riprodotto).** Durante la prima esecuzione su HW il core 1
  era rimasto nella bootrom (PC=0xda, il ciclo in cui aspetta di essere avviato) mentre
  FreeRTOS gli aveva già assegnato il task dei test: sulla seriale si vedeva solo il banner.
  Era appena stata chiusa una sessione Cortex-Debug e c'erano stati reset inviati al suo
  openocd. In seguito il problema non si è più presentato: 15 `reset run` consecutivi, il
  vecchio comando `program ... verify reset exit`, sessione Cortex-Debug chiusa normalmente
  e con `kill -9`. Ogni volta entrambi i core eseguono FreeRTOS. Causa non identificata. Il
  task "Flash" ora usa `init; reset halt; program ... verify; reset halt; resume; exit`
  (innocuo, ma non dimostrato necessario).
- **"Run Project" (`picotool load -fx`)** non funziona con questo setup: sull'USB è visibile
  solo la Debug Probe, e il firmware non espone l'interfaccia di reset USB
  (`pico_enable_stdio_usb 0`), quindi `-f` non può riavviare la scheda in BOOTSEL.
- I sorgenti di `osal-rs-tests` non sono tra le dipendenze CMake del target Rust: una
  modifica lì non ricompila il firmware finché non si tocca un file tracciato.
