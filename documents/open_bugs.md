# Bug aperti — Hi Happy Garden firmware

Stato al 2026-10-05. I bug sono emersi durante la scrittura della suite di test on-target
(`-DHHG_TESTS=ON`). Dove indicato, la suite contiene un test che oggi **fallisce** e che
passerà quando il bug sarà corretto. I bug marcati *senza test* farebbero andare in panic
la suite (con `panic = "abort"` si fermerebbe tutto), quindi non sono coperti da un test.

> **Esecuzione su HW del 2026-10-03 (Pico 2 W):** 137 test passati, 11 falliti. Gli 11
> fallimenti corrispondono ai bug 1–7, 11, 12 e 17 di questo documento: nessun altro
> test fallisce. La suite osal-rs passa per intero.
>
> **Aggiornamento 2026-10-03:** i bug 1, 2 e 3 sono corretti nel codice (il firmware compila)
> e spostati in [Risolti](#risolti). I relativi test non sono ancora stati rieseguiti su HW.
>
> **Aggiornamento 2026-10-04:** corretti nel codice anche i bug 4 e 6 (la suite compila con
> `--features tests`), spostati in [Risolti](#risolti). Per il bug 5 è corretta solo la parte
> `zn` di `AT+SCH`: `AT+ZN` resta aperto.
>
> **Esecuzione su HW del 2026-10-04 (Pico 2 W):** 144 test passati, 6 falliti. Confermati
> corretti i bug 1, 2, 3, 4 e 6. Restano i fallimenti dei bug 5, 7, 11, 12 e 17, più
> `ScheduleTests::test_set_bounds`: il test accettava ancora `st,2`, mentre la guardia era già
> stata portata a `st ≤ ACTIVE`. Il test è stato allineato dopo l'esecuzione e va rieseguito.
> Il log ha mostrato anche il nuovo bug 18, che nessun test fa fallire.
>
> **Esecuzione su HW del 2026-10-04, seconda (Pico 2 W, dopo `23dbe84` e osal-rs `91947f0`):**
> 145 test passati, 5 falliti. Restano solo i fallimenti dei bug 5, 7, 11, 12 e 17.
> `ScheduleTests::test_set_bounds` ora passa. Il bug 18 è corretto: nel log non ci sono più
> errori di deserializzazione di `zones.json`/`schedules.json`. Bug 18 spostato in
> [Risolti](#risolti).
>
> **Esecuzione su HW del 2026-10-05 (Pico 2 W, dopo `d226534`, `af165eb` e `2c858bc`):**
> 148 test passati, 3 falliti. Restano solo i fallimenti dei bug 11, 12 e 17.
> Confermati corretti i bug 5 (`ZoneTests::test_set_unknown_relay`) e 7
> (`DateTimeTests::test_dst_eu_fall_back`, più il nuovo `test_dst_end_at_midnight`),
> spostati in [Risolti](#risolti).
>
> **Aggiornamento 2026-10-05:** il bug 8 è corretto nel codice (il firmware compila) e spostato in
> [Risolti](#risolti), con il nuovo test `FilesystemTests::test_read_error`, non ancora eseguito su HW.
> La descrizione originale era imprecisa: vedi la voce in [Risolti](#risolti) e il bug 14.
> Corretto nel codice anche il bug 9 (spostato in [Risolti](#risolti)), con il test
> `ParserTests::test_uart_two_lines_in_one_buffer`, non ancora eseguito su HW.
>
> **Aggiornamento 2026-10-06:** i bug 10, 11 e 12 sono corretti nel codice e spostati in
> [Risolti](#risolti).
>
> **Esecuzione su HW del 2026-10-06 (Pico 2 W, dopo `9aa219a`, `3f3c4c6` e la correzione del bug 12):**
> 153 test passati, 1 fallito. Resta solo il fallimento del bug 17 (`ZoneTests::test_query`).
> Confermati corretti su HW i bug 8 (`FilesystemTests::test_read_error`), 9
> (`ParserTests::test_uart_two_lines_in_one_buffer`), 10
> (`HardwareErrorTests::test_flags_try_from_invalid`, `test_flags_roundtrip`), 11
> (`SignalsTests::test_error_flags`) e 12 (`DateTimeTests::test_negative_timestamp`).

## Riepilogo

| #  | Gravità | Area | Titolo | Test |
|----|---------|------|--------|------|
| 13 | Bassa   | Filesystem | Errore di scrittura littlefs ignorato | *senza test* |
| 14 | Bassa   | Utils | Fallback di `deserialize_file` legge un file aperto in sola scrittura | *senza test* |
| 15 | Bassa   | Config | `AT+DST` non valida i range | *senza test* |
| 16 | Bassa   | Build | Documentazione di `HHG_DEFAULT_WIFI_AUTH` non coerente con `Auth` | *senza test* |
| 17 | Media   | Sprinkler | `AT+ZN?` restituisce solo l'ultima zona | `ZoneTests::test_query` |

---

## 13. Errore di scrittura littlefs ignorato

**File:** [main/src/drivers/pico/flash.rs:96](../main/src/drivers/pico/flash.rs#L96) e
[filesystem.rs:272](../main/src/drivers/filesystem.rs#L272)

`file_write` restituisce `Ok` anche quando `hhg_flash_write` è negativo (flash piena, I/O);
`File::write` salva poi `ret as u32` in `size`. Un salvataggio di configurazione fallito
viene quindi segnalato come riuscito.

**Correzione proposta:** propagare `Err(Error::ReturnWithCode(written))` se `written < 0`.

## 14. Fallback di `deserialize_file` legge un file aperto in sola scrittura

**File:** [main/src/apps/utils.rs:56](../main/src/apps/utils.rs#L56)

Su `-2` (NOENT) il file viene riaperto con `WRONLY | CREAT` e poi letto. littlefs non
restituisce un errore ma esegue `LFS_ASSERT` (attivo in Debug), quindi il firmware si sarebbe
fermato. Dal 2026-10-05 `hhg_flash_read` intercetta il caso e restituisce `LFS_ERR_BADF` (vedi #8
tra i risolti): ora la lettura fallisce senza crash e si passa ai default. Il ramo è raro, perché
`RDWR | CREAT` non dovrebbe mai dare NOENT, ma resta sbagliato.

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

**Output su HW (2026-10-04):**
```
[ZoneTests] test_query FAILED: Unhandled error owned: src/apps/sprinkler/zone/tests.rs:122: body != expected ("3,3,\"Relay 3\"\r\n" != "0,0,\"Relay 0\"\r\n1,1,\"Relay 1\"\r\n2,2,\"Relay 2\"\r\n3,3,\"Relay 3\"\r\n")
```

---

## Risolti

- **#1 Maschera dei mesi spostata di uno** (Sprinkler, alta). `From<Month> for u8` ora
  restituisce 1..12, come `DateTime::month`; `Month::map` usa il bit `idx - 1`.
  Test: `SprinklerTests::test_months`, passato su HW il 2026-10-04.
- **#2 Programmi `UNACTIVE` eseguibili** (Sprinkler, alta). `executable()` ora restituisce
  `false` se lo stato non è `ACTIVE`. Test: `SprinklerTests::test_inactive_not_executable`,
  passato su HW il 2026-10-04. Resta aperto un aspetto collegato: `Sprinkler::check` mette il
  programma in `RUN` e nessuno lo riporta ad `ACTIVE` né azzera `DISBURSEMENT_IN_PROGRESS`.
  Va gestito quando si implementa l'erogazione.
- **#3 Zone di default tutte su `Relay0`, senza descrizione** (Sprinkler, alta).
  `ZoneController` e `ScheduleController` hanno ora un `Default` scritto a mano: zone
  `Relay0..Relay3` con descrizione `"Relay N"` e peso = indice, schedule con descrizione
  `"Schedule N"`. La preparazione di `SHARED` in `init` è stata rimossa. Test:
  `ZoneTests::test_zones_cover_all_relays`, `test_zones_have_description`, passati su HW il 2026-10-04.
  Sulle schede già inizializzate bisogna cancellare `zones.json`/`schedules.json`.
  **Attenzione:** in `ZoneController::init` la chiamata a `deserialize_file` è di nuovo attiva,
  ma in [`Hardware::init_fs`](../main/src/drivers/pico/hardware.rs#L261) c'è ancora
  `remove_recursive("/")` (verificato il 2026-10-04): a ogni avvio il filesystem viene
  cancellato, quindi nessuna configurazione salvata sopravvive al riavvio. Va rimosso o
  limitato alla build dei test.
- **#4 `AT+ZN`: le modifiche in staging si perdevano** (Sprinkler, media). `set` copia la
  zona in `ZONE_TMP` solo se lo staging è vuoto o riguarda un altro relè, quindi più `set`
  sulla stessa zona si accumulano fino all'`exec`, come in `AT+SCH`.
  Test: `ZoneTests::test_set_many_then_exec`, passato su HW il 2026-10-04.
- **#6 `AT+SCH` accettava valori fuori range** (Sprinkler, media). `set` ora rifiuta con
  `InvalidArgs`: indice `>= ScheduleController::SIZE`, `mi > 60`, `hr > 24`, `dy > 0x7F`,
  `mo > 0x0FFF`, relè `zn >= ZoneController::SIZE`, `st > ACTIVE` (`RUN` è uno stato di esecuzione e non si imposta via AT). In `zn` 0 minuti indica una
  zona disattivata ed è accettato. Poiché l'indice è validato in `set`, `SCHEDULE_TMP` non può
  più contenere un indice fuori range; il controllo in `exec` resta come seconda difesa.
  Test: `ScheduleTests::test_set_out_of_range`, `test_set_bounds` (0 e massimo accettati),
  `test_set_bad_index`, `test_exec_bad_index`: passati su HW il 2026-10-04. `test_set_bounds`
  (allineato a `st ≤ ACTIVE`) è passato nella seconda esecuzione dello stesso giorno.
- **#5 `AT+ZN` accettava un relè inesistente** (Sprinkler, media). `ZoneRelay::from(u8)`
  mappa ogni valore sconosciuto su `Relay0`, quindi `AT+ZN=7,ds,X` modificava la zona 0.
  In `d226534` `ZoneController::set` rifiuta con `InvalidArgs` un `zone_relay >= ZoneController::SIZE`
  prima della conversione, come già il `zn` di `AT+SCH` (#6). `ZoneRelay::from` resta
  permissivo: un `TryFrom<u8>` eviterebbe il problema anche per i valori letti da JSON.
  Test: `ZoneTests::test_set_unknown_relay`, passato su HW il 2026-10-05.
- **#7 Fine ora legale un'ora in ritardo** (DateTime, media). `END_HOUR` è espresso in ora
  legale (regola UE: 03:00 CEST = 02:00 CET), mentre `is_daylight_saving_time` lavora sull'ora
  solare locale: il 27/10/2024 alle 01:30 UTC il firmware mostrava 03:30 invece di 02:30.
  In `af165eb` la fine è stata anticipata di un'ora (`end_hour - 1`), ma su `u8`: con
  `end_hour = 0` andava in underflow (panic in Debug, `255` in Release, cioè ora legale attiva
  per tutto il giorno di fine). In `2c858bc` il confronto usa le ore dall'inizio dell'anno
  (`hour_of_year`), così l'ora sottratta passa correttamente al giorno o al mese precedente.
  Test: `DateTimeTests::test_dst_eu_fall_back` e `test_dst_end_at_midnight` (fine il 1° novembre
  alle 00:00), passati su HW il 2026-10-05.
- **#8 `file_read` andava in panic su errore littlefs** (Filesystem, media). `hhg_flash_read`
  era dichiarata con ritorno `lfs_size_t` in C e `LfsSize` (`u32`) in Rust: un codice d'errore
  negativo diventava una lunghezza enorme e `buffer[..len]` andava in panic. Ora il ritorno è
  `lfs_ssize_t`/`LfsSsize` (`i32`) da entrambe le parti, e `file_read` restituisce
  `Err(Error::ReturnWithCode(len))` se `len < 0`
  ([flash.rs:129](../main/src/drivers/pico/flash.rs#L129)). Lo stesso cambio di tipo è stato fatto
  su `hhg_flash_write`, ma il controllo in `file_write` manca ancora (#13).
  **Correzione della descrizione originale:** l'esempio "`LFS_ERR_BADF` su un file aperto in sola
  scrittura" era sbagliato. In littlefs v2.11.2, `lfs_file_read` su un file senza `LFS_O_RDONLY`
  esegue `LFS_ASSERT` invece di restituire un errore, e in Debug l'assert ferma il firmware. I valori
  negativi arrivavano solo da `lfs_file_flush` o da errori della flash (`LFS_ERR_IO`, `LFS_ERR_CORRUPT`).
  Per questo `hhg_flash_read` ([hhg-lfs-wrapper.c](../src/pico/hhg-lfs-wrapper.c)) ora controlla il
  flag e restituisce `LFS_ERR_BADF`, come `read` POSIX su un descrittore in sola scrittura.
  `hhg_flash_write` ha lo stesso assert per i file aperti in sola lettura e non è ancora protetta.
  Test: `FilesystemTests::test_read_error`, che legge un file aperto in `WRONLY` sia da `FILE_FN.read`
  sia da `File::read` e si aspetta `ReturnWithCode(LFS_ERR_BADF)`. Passato su HW il 2026-10-06.
- **#9 Panic con più righe in un solo `on_receive`** (Parser, media). Il protocollo prevede un
  comando per ogni `on_receive`, perché a ogni comando corrisponde una risposta: più righe nello
  stesso buffer non sono un caso d'uso valido. Il task del parser però andava in panic: `SOURCE`
  veniva impostata solo dopo aver accodato i byte e azzerata dopo ogni riga, quindi alla seconda
  riga `access_static_option!(SOURCE)` trovava `None`. Lo stesso valeva per `MQTT_CHANNEL`, mai
  impostato. Ora:
  - `on_receive` imposta `SOURCE` **prima** di accodare e rifiuta con `Error::InvalidType` i dati
    di un'altra sorgente finché il comando in corso non è stato letto;
  - il task prende la sorgente (`take()`) appena la riga è completa: la libera su ogni percorso,
    compreso il `KO` di una sessione aperta da un'altra sorgente, che prima lasciava `SOURCE`
    impostata e avrebbe bloccato per sempre l'altro canale;
  - una riga senza sorgente o senza canale viene scartata con un log di errore invece del panic.
  Test: `ParserTests::test_uart_two_lines_in_one_buffer` passa due righe in una sola chiamata con
  `Source::Uart`, si aspetta una sola risposta e verifica che il comando successivo funzioni.
  Passato su HW il 2026-10-06. `MQTT_CHANNEL` va ancora impostato quando si collega MQTT: oggi i comandi MQTT
  verrebbero scartati con `Channel is not set`.
- **#10 `HardwareErrorFlag::from` andava in panic su valore sconosciuto** (Drivers, bassa).
  Aggiunta la variante `None = 0x00`, come per `ErrorFlag`/`StatusFlag`: `From<u32>` restituisce
  `None` per 0, per le combinazioni di bit di `HardwareErrorSignal::get()` e per i valori fuori
  range, invece di chiamare `panic!`. Test: `HardwareErrorTests::test_flags_try_from_invalid`
  (nuovo) e `test_flags_roundtrip`, passati su HW il 2026-10-06.
- **#11 `ErrorFlag::from(0x08)` non restituiva `DisplayHeader`** (Signals, bassa). Aggiunto il
  caso `0x08 => DisplayHeader` nel `match`, quindi la conversione fa il roundtrip. Test:
  `SignalsTests::test_error_flags`, fallito su HW il 2026-10-04 e il 2026-10-05, passato il
  2026-10-06 dopo la correzione.
- **#12 `wday` errato per timestamp negativi** (DateTime, bassa). In `from_timestamp_raw` il
  giorno della settimana si calcola ora con `timestamp.div_euclid(SECONDS_PER_DAY)` invece di
  `/`, che troncava verso zero: per `-1` (1969-12-31) usciva giovedì invece di mercoledì.
  Test: `DateTimeTests::test_negative_timestamp`, fallito su HW il 2026-10-04 e il 2026-10-05,
  passato il 2026-10-06 dopo la correzione.
- **#18 `zones.json`/`schedules.json` salvati non si ricaricavano** (Serde, alta). Il derive di
  osal-rs-serde non era simmetrico per le tuple struct: `Serialize` scriveva un oggetto con i
  campi `"0"`, `"1"`, …, mentre `Deserialize` leggeva i campi direttamente dalla radice. Il JSON
  salvato non si rileggeva, venivano caricati i default e il file veniva sovrascritto.
  Corretto su due fronti: in osal-rs (`44b754a`) `Deserialize` ora entra nell'oggetto e legge
  `"0"`, `"1"`, …; in `23dbe84` `ZoneController` e `ScheduleController` sono tornati a struct
  con campo nominato (`zones`, `schedules`). Nell'esecuzione su HW del 2026-10-04 (la seconda)
  non compare più `Failed to deserialize JSON` dopo il `sv` di zone e schedule.
  **Da fare:** `test_save_reload` (zone e schedule) resta poco significativo: salva i dati
  ripristinati, cioè i default, quindi passerebbe anche con una lettura fallita. Andrebbe
  modificato un valore prima del salvataggio, confrontato dopo il ricaricamento e poi ripristinato.
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
- `AT+SCH` e `AT+ZN` caricano in staging la copia salvata già al primo `set`, anche se il comando
  poi fallisce (es. `AT+SCH=1,mi,99`). Poiché i default hanno una descrizione, la copia risulta
  "modificata" e un `exec` successivo risponde OK riapplicando valori invariati invece di
  `No modify applied`. Innocuo, ma la risposta è fuorviante.
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
