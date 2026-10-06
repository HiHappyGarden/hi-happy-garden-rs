/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/


//! Config tests: defaults, JSON persistence and the `AT+CNF`, `AT+DST`,
//! `AT+WIFI`, `AT+NTP` handlers, called directly (the parser plumbing has
//! its own tests).
//!
//! Every test runs on a copy of the live configuration that is put back
//! afterwards; the `sv` sub-command is never sent, the only flash write is
//! `test_save_reload`, which saves the configuration unchanged.

use alloc::format;

use at_parser_rs::Args;
use at_parser_rs::context::AtContext;
use osal_rs::utils::Result;

use super::defaults::*;
use super::{Config, DaylightSavingTime, MUTEX, NtpConfig, WifiConfig};
use crate::apps::parser::Parser;
use crate::apps::test_helpers::{from_json, to_json, at_body, at_error, logged, with_test_user, TEST_PASSWORD};
use crate::apps::utils::deserialize_file;
use crate::drivers::date_time::DateTime;
use crate::drivers::platform::FS_CONFIG_DIR;
use crate::drivers::wifi::Auth;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};

const TAG: &str = "ConfigTests";

/// Runs `test` on the live configuration and restores it afterwards,
/// re-applying the restored values to DateTime, Wifi and Network.
fn with_config_restored(test: impl FnOnce(&'static mut Config) -> Result<()>) -> Result<()> {
    let backup = *Config::shared();
    let ret = test(Config::shared());
    let config = Config::shared();
    *config = backup;
    config.apply_locale();
    config.apply_daylight_saving_time();
    config.apply_ntp();
    config.apply_wifi();
    ret
}

fn json(config: &Config) -> Result<alloc::string::String> {
    to_json(config)
}

fn test_defaults() -> Result<()> {
    let config = Config::default();
    test_assert_eq!(config.timezone, DEFAULT_TIMEZONE);
    test_assert!(config.serial.is_empty());
    test_assert_eq!(config.ntp.port, DEFAULT_NTP_PORT);
    test_assert_eq!(config.ntp.msg_len, DEFAULT_NTP_MSG_LEN);
    test_assert_eq!(config.ntp.server.as_str(), DEFAULT_NTP_SERVER);
    test_assert_eq!(config.wifi.enabled, DEFAULT_WIFI_ENABLED);
    test_assert_eq!(config.daylight_saving_time.enabled, DEFAULT_DAYLIGHT_SAVING_ENABLED);
    test_assert_eq!(config.daylight_saving_time.start_month, DEFAULT_DAYLIGHT_SAVING_START_MONTH);
    test_assert_eq!(config.daylight_saving_time.end_month, DEFAULT_DAYLIGHT_SAVING_END_MONTH);
    test_assert!(!config.session.is_set_user_local());
    Ok(())
}

fn test_json_roundtrip() -> Result<()> {
    let first = json(Config::shared())?;
    let parsed: Config = from_json(&first)?;
    test_assert_eq!(json(&parsed)?, first);
    Ok(())
}

fn test_json_has_no_plain_password() -> Result<()> {
    with_test_user(|| {
        let text = json(Config::shared())?;
        test_assert!(!text.contains(TEST_PASSWORD), "user password stored in clear");
        Ok(())
    })
}

fn test_save_reload() -> Result<()> {
    let expected = json(Config::shared())?;
    Config::save()?;
    let reloaded: Config = deserialize_file(unsafe { &*&raw const MUTEX }, TAG, FS_CONFIG_DIR, Config::FILE_NAME)?;
    test_assert_eq!(json(&reloaded)?, expected);
    Ok(())
}

fn test_cnf_query_and_test() -> Result<()> {
    let config = Config::shared();
    let expected = format!("\"{}\",{}", config.get_serial().as_str(), config.get_timezone());
    test_assert_eq!(at_body(config.query(Config::AT_RESP))?, expected);
    test_assert!(!at_body(config.test(Config::AT_RESP))?.is_empty());
    Ok(())
}

fn test_set_requires_login() -> Result<()> {
    with_config_restored(|config| {
        let timezone = config.get_timezone();
        test_assert_eq!(at_error(config.set(Config::AT_RESP, Args { raw: "tz,1" }))?, Parser::NOT_LOGGED_RESPONSE);
        test_assert_eq!(config.get_timezone(), timezone);

        test_assert_eq!(at_error(config.get_daylight_saving_time().set(DaylightSavingTime::AT_RESP, Args { raw: "en,1" }))?, Parser::NOT_LOGGED_RESPONSE);
        test_assert_eq!(at_error(config.get_wifi_config().set(WifiConfig::AT_RESP, Args { raw: "a,b,3,1" }))?, Parser::NOT_LOGGED_RESPONSE);
        test_assert_eq!(at_error(config.get_ntp_config_mut().set(NtpConfig::AT_RESP, Args { raw: "a,1,48" }))?, Parser::NOT_LOGGED_RESPONSE);
        Ok(())
    })
}

fn test_cnf_set_timezone() -> Result<()> {
    with_config_restored(|config| logged(|| {
        config.get_daylight_saving_time().set_enabled(false);
        config.apply_daylight_saving_time();

        test_assert!(at_body(config.set(Config::AT_RESP, Args { raw: "tz,120" }))?.is_empty());
        test_assert_eq!(config.get_timezone(), 120);

        // Applied to the clock right away, no reboot needed
        let dt = DateTime::from_timestamp_locale(0, true)?;
        test_assert!(dt.is_apply_timezone());
        test_assert_eq!(dt.hour, 2);

        test_assert!(at_body(config.set(Config::AT_RESP, Args { raw: "tz,-330" }))?.is_empty());
        test_assert_eq!(config.get_timezone(), -330);
        Ok(())
    }))
}

fn test_cnf_set_serial() -> Result<()> {
    with_config_restored(|config| logged(|| {
        test_assert!(at_body(config.set(Config::AT_RESP, Args { raw: "sn,HHG-TEST-0001" }))?.is_empty());
        test_assert_eq!(config.get_serial().as_str(), "HHG-TEST-0001");
        test_assert_eq!(at_body(config.query(Config::AT_RESP))?, format!("\"HHG-TEST-0001\",{}", config.get_timezone()));

        test_assert_eq!(at_error(config.set(Config::AT_RESP, Args { raw: "sn,12345678901234567" }))?, "serial max len 16");
        test_assert_eq!(config.get_serial().as_str(), "HHG-TEST-0001");
        Ok(())
    }))
}

fn test_cnf_invalid_args() -> Result<()> {
    with_config_restored(|config| logged(|| {
        for raw in ["", "xx,1", "tz", "tz,abc", "tz,99999", "sn"] {
            test_assert_eq!(at_error(config.set(Config::AT_RESP, Args { raw }))?, "InvalidArgs");
        }
        Ok(())
    }))
}

fn test_dst() -> Result<()> {
    with_config_restored(|config| logged(|| {
        let dst = config.get_daylight_saving_time();
        for raw in ["smo,4", "sdy,1", "shr,3", "emo,9", "edy,30", "ehr,2", "en,1"] {
            test_assert!(at_body(dst.set(DaylightSavingTime::AT_RESP, Args { raw }))?.is_empty(), "{raw} rejected");
        }
        test_assert_eq!(at_body(dst.query(DaylightSavingTime::AT_RESP))?, "4,1,3,9,30,2,1");
        test_assert!(dst.is_enabled());

        test_assert!(at_body(dst.set(DaylightSavingTime::AT_RESP, Args { raw: "en,0" }))?.is_empty());
        test_assert!(!dst.is_enabled());

        for raw in ["xx,1", "smo", "smo,300", "en,x"] {
            test_assert_eq!(at_error(dst.set(DaylightSavingTime::AT_RESP, Args { raw }))?, "InvalidArgs");
        }

        // Range bounds (open_bugs #15): 255 is "last Sunday of the month"
        for raw in ["smo,1", "smo,12", "sdy,1", "sdy,31", "sdy,255", "shr,0", "shr,23",
                    "emo,1", "emo,12", "edy,1", "edy,31", "edy,255", "ehr,0", "ehr,23"] {
            test_assert!(at_body(dst.set(DaylightSavingTime::AT_RESP, Args { raw }))?.is_empty(), "{raw} rejected");
        }
        test_assert_eq!(at_body(dst.query(DaylightSavingTime::AT_RESP))?, "12,255,23,12,255,23,0");
        for raw in ["smo,0", "smo,13", "sdy,0", "sdy,32", "sdy,254", "shr,24",
                    "emo,0", "emo,13", "edy,0", "edy,32", "edy,254", "ehr,24"] {
            test_assert_eq!(at_error(dst.set(DaylightSavingTime::AT_RESP, Args { raw }))?, "InvalidArgs", "{raw} accepted");
        }
        // Rejected values leave the configuration untouched
        test_assert_eq!(at_body(dst.query(DaylightSavingTime::AT_RESP))?, "12,255,23,12,255,23,0");
        Ok(())
    }))
}

fn test_wifi() -> Result<()> {
    with_config_restored(|config| logged(|| {
        let wifi = config.get_wifi_config();
        test_assert!(at_body(wifi.set(WifiConfig::AT_RESP, Args { raw: "\"Garden Net\",\"s3cret,pwd\",5,1" }))?.is_empty());
        test_assert_eq!(wifi.get_ssid().as_str(), "Garden Net");
        test_assert_eq!(wifi.get_password().as_str(), "s3cret,pwd");
        test_assert_eq!(wifi.get_auth(), Auth::Wpa3);
        test_assert!(wifi.is_enabled());

        let query = at_body(wifi.query(WifiConfig::AT_RESP))?;
        test_assert_eq!(query, "\"Garden Net\",5,1");
        test_assert!(!query.contains("s3cret"), "password leaked by the query");

        test_assert_eq!(at_error(wifi.set(WifiConfig::AT_RESP, Args { raw: "123456789012345678901234567890123,p,3,1" }))?, "ssid max len 32");
        test_assert_eq!(at_error(wifi.set(WifiConfig::AT_RESP, Args { raw: "s,123456789012345678901234567890123,3,1" }))?, "password max len 32");
        test_assert_eq!(at_error(wifi.set(WifiConfig::AT_RESP, Args { raw: "s,p,3" }))?, "InvalidArgs");
        test_assert_eq!(at_error(wifi.set(WifiConfig::AT_RESP, Args { raw: "s,p,x,1" }))?, "InvalidArgs");
        // Rejected commands leave the previous values in place
        test_assert_eq!(wifi.get_ssid().as_str(), "Garden Net");
        Ok(())
    }))
}

fn test_ntp() -> Result<()> {
    with_config_restored(|config| logged(|| {
        let ntp = config.get_ntp_config_mut();
        test_assert!(at_body(ntp.set(NtpConfig::AT_RESP, Args { raw: "\"time.example.org\",1123,48" }))?.is_empty());
        test_assert_eq!(at_body(ntp.query(NtpConfig::AT_RESP))?, "\"time.example.org\",1123,48");
        test_assert_eq!(config.get_ntp_config().get_port(), 1123);

        let long = "a".repeat(65);
        let raw = format!("{long},123,48");
        test_assert_eq!(at_error(config.get_ntp_config_mut().set(NtpConfig::AT_RESP, Args { raw: &raw }))?, "server max len 64");
        test_assert_eq!(at_error(config.get_ntp_config_mut().set(NtpConfig::AT_RESP, Args { raw: "s,70000,48" }))?, "InvalidArgs");
        Ok(())
    }))
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_defaults,
        test_json_roundtrip,
        test_json_has_no_plain_password,
        test_save_reload,
        test_cnf_query_and_test,
        test_set_requires_login,
        test_cnf_set_timezone,
        test_cnf_set_serial,
        test_cnf_invalid_args,
        test_dst,
        test_wifi,
        test_ntp,
    );
}
