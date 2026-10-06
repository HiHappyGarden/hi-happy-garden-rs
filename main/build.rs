use sha2::{Digest, Sha256};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

fn unquote(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

fn rust_string_literal(value: &str) -> String {
    format!("{:?}", unquote(value))
}

/// Path of the secrets file, at the repository root next to the main CMakeLists.txt.
fn secrets_path() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("..").join("secrets.cmake")
}

/// Parses the `set(NAME value)` lines of secrets.cmake.
///
/// Used as fallback when cargo is not invoked by CMake (e.g. rust-analyzer, plain `cargo check`).
fn secrets_file() -> &'static Vec<(String, String)> {
    static SECRETS: OnceLock<Vec<(String, String)>> = OnceLock::new();
    SECRETS.get_or_init(|| {
        let content = std::fs::read_to_string(secrets_path()).unwrap_or_default();
        content
            .lines()
            .map(str::trim)
            .filter_map(|line| line.strip_prefix("set("))
            .filter_map(|rest| rest.rfind(')').map(|end| rest[..end].trim()))
            .filter_map(|args| {
                let (name, value) = args.split_once(char::is_whitespace)?;
                Some((name.to_string(), unquote(value).to_string()))
            })
            .collect()
    })
}

/// Reads a configuration variable from the environment (set by CMake), falling back to secrets.cmake.
fn hhg_var(var_name: &str) -> Result<String, env::VarError> {
    env::var(var_name).or_else(|err| {
        secrets_file()
            .iter()
            .find(|(name, _)| name == var_name)
            .map(|(_, value)| value.clone())
            .ok_or(err)
    })
}

fn env_string_literal(var_name: &str, default: &str) -> String {
    let value = hhg_var(var_name).unwrap_or_else(|_| default.to_string());
    rust_string_literal(&value)
}

/// Reads a secret that must be provided by CMake (from secrets.cmake), without default.
///
/// Panics if the variable is missing or empty, stopping the build.
fn env_required(var_name: &str) -> String {
    let value = hhg_var(var_name).unwrap_or_default();
    let value = unquote(&value);
    if value.is_empty() {
        panic!("{} is not set: define it in secrets.cmake (see secrets.cmake.example)", var_name);
    }
    value.to_string()
}

fn parse_bool(s: &str) -> bool {
    let cleaned = s.trim().trim_matches('"').to_lowercase();
    matches!(cleaned.as_str(), "true" | "1" | "on" | "yes")
}

/// Locates the source tree of a C dependency of the emulator.
///
/// `env_var` wins when set; otherwise the copy CMake's FetchContent left in
/// `build/_deps` (firmware build) or `build-emulator/_deps` (see
/// `scripts/emulator-deps.sh`) is used, so no extra download is needed.
fn emulator_dep_dir(env_var: &str, fetch_name: &str, probe: &str) -> PathBuf {
    println!("cargo:rerun-if-env-changed={}", env_var);
    if let Ok(dir) = env::var(env_var) {
        return PathBuf::from(dir);
    }

    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("..");
    ["build", "build-emulator"]
        .iter()
        .map(|build| root.join(build).join("_deps").join(format!("{}-src", fetch_name)))
        .find(|dir| dir.join(probe).exists())
        .unwrap_or_else(|| panic!(
            "{} sources not found: set {} or run scripts/emulator-deps.sh",
            fetch_name, env_var
        ))
}

/// Builds the host side of the emulator: littlefs behind the `hhg_flash_*`
/// wrapper of `src/emulator` and cJSON for `cjson-bindings`, both statically
/// linked so the emulator does not depend on system libraries.
fn build_emulator_c_layer() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("..");
    let littlefs = emulator_dep_dir("LITTLEFS_DIR", "littlefs", "lfs.c");
    let cjson = emulator_dep_dir("CJSON_SRC_DIR", "cjson", "cJSON.c");
    let lfs_wrapper = root.join("src").join("emulator").join("hhg-lfs-wrapper.c");

    println!("cargo:rerun-if-changed={}", lfs_wrapper.display());

    cc::Build::new()
        .file(littlefs.join("lfs.c"))
        .file(littlefs.join("lfs_util.c"))
        .file(&lfs_wrapper)
        .include(&littlefs)
        // littlefs debug/warn traces would interleave with the firmware log
        .define("LFS_NO_DEBUG", None)
        .define("LFS_NO_WARN", None)
        .warnings(false)
        .cargo_metadata(false)
        .compile("hhg_emulator_lfs");

    cc::Build::new()
        .file(cjson.join("cJSON.c"))
        .file(cjson.join("cJSON_Utils.c"))
        .include(&cjson)
        .warnings(false)
        .cargo_metadata(false)
        .compile("hhg_emulator_cjson");

    // Whole archive: cJSON is referenced by `cjson-bindings`, which the linker
    // meets after this crate, and a plain static archive would already have
    // been skipped by then
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static:+whole-archive=hhg_emulator_lfs");
    println!("cargo:rustc-link-lib=static:+whole-archive=hhg_emulator_cjson");
}

fn main() {
    if env::var_os("CARGO_FEATURE_EMULATOR").is_some() {
        build_emulator_c_layer();
    }

    // Read configuration from environment variables set by CMake
    let default_wifi_ssid = env_string_literal("HHG_DEFAULT_WIFI_SSID", "");
    let default_wifi_password = env_string_literal("HHG_DEFAULT_WIFI_PASSWORD", "");
    let default_wifi_auth = hhg_var("HHG_DEFAULT_WIFI_AUTH").unwrap_or_else(|_| "3".to_string()).parse::<u8>().unwrap_or(3);
    let default_wifi_enabled = parse_bool(&hhg_var("HHG_DEFAULT_WIFI_ENABLED").unwrap_or_else(|_| "false".to_string()));
    let default_timezone = hhg_var("HHG_DEFAULT_TIMEZONE").unwrap_or_else(|_| "60".to_string()).parse::<i16>().unwrap_or(60);
    let default_daylight_saving_enabled = parse_bool(&hhg_var("HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED").unwrap_or_else(|_| "false".to_string()));
    let default_daylight_saving_start_month = hhg_var("HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH").unwrap_or_else(|_| "3".to_string()).parse::<u8>().unwrap_or(3);
    let default_daylight_saving_start_day = hhg_var("HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY").unwrap_or_else(|_| "255".to_string()).parse::<u8>().unwrap_or(255);
    let default_daylight_saving_start_hour = hhg_var("HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR").unwrap_or_else(|_| "2".to_string()).parse::<u8>().unwrap_or(2);
    let default_daylight_saving_end_month = hhg_var("HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH").unwrap_or_else(|_| "10".to_string()).parse::<u8>().unwrap_or(10);
    let default_daylight_saving_end_day = hhg_var("HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY").unwrap_or_else(|_| "255".to_string()).parse::<u8>().unwrap_or(255);
    let default_daylight_saving_end_hour = hhg_var("HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR").unwrap_or_else(|_| "3".to_string()).parse::<u8>().unwrap_or(3);
    let default_ntp_msg_len = hhg_var("HHG_DEFAULT_NTP_MSG_LEN").unwrap_or_else(|_| "48".to_string()).parse::<u16>().unwrap_or(48);
    let default_ntp_port = hhg_var("HHG_DEFAULT_NTP_PORT").unwrap_or_else(|_| "123".to_string()).parse::<u16>().unwrap_or(123);
    let default_ntp_server = env_string_literal("HHG_DEFAULT_NTP_SERVER", "0.europe.pool.ntp.org");
    let hhg_aes_key_salt = rust_string_literal(&env_required("HHG_AES_KEY_SALT"));
    let hhg_aes_iv_salt = rust_string_literal(&env_required("HHG_AES_IV_SALT"));
    let system_user_email = rust_string_literal(&env_required("HHG_SYSTEM_USER_EMAIL"));
    let raw_password = env_required("HHG_SYSTEM_USER_PASSWORD");
    let mut hasher = Sha256::new();
    hasher.update(raw_password.as_bytes());
    let system_user_password = format!("{:?}", format!("{:x}", hasher.finalize()));

    // Generate defaults.rs file
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("defaults.rs");
    let mut f = File::create(&dest_path).unwrap();
    
    writeln!(f, "// Auto-generated by build.rs from CMake configuration").unwrap();
    writeln!(f, "").unwrap();
    writeln!(f, "pub const DEFAULT_WIFI_SSID: &str = {};", default_wifi_ssid).unwrap();
    writeln!(f, "pub const DEFAULT_WIFI_PASSWORD: &str = {};", default_wifi_password).unwrap();
    writeln!(f, "pub const DEFAULT_WIFI_AUTH: u8 = {};", default_wifi_auth).unwrap();
    writeln!(f, "pub const DEFAULT_WIFI_ENABLED: bool = {};", default_wifi_enabled).unwrap();
    writeln!(f, "pub const DEFAULT_TIMEZONE: i16 = {};", default_timezone).unwrap();
    writeln!(f, "pub const DEFAULT_DAYLIGHT_SAVING_ENABLED: bool = {};", default_daylight_saving_enabled).unwrap();
    writeln!(f, "pub const DEFAULT_DAYLIGHT_SAVING_START_MONTH: u8 = {};", default_daylight_saving_start_month).unwrap();
    writeln!(f, "pub const DEFAULT_DAYLIGHT_SAVING_START_DAY: u8 = {};", default_daylight_saving_start_day).unwrap();
    writeln!(f, "pub const DEFAULT_DAYLIGHT_SAVING_START_HOUR: u8 = {};", default_daylight_saving_start_hour).unwrap();
    writeln!(f, "pub const DEFAULT_DAYLIGHT_SAVING_END_MONTH: u8 = {};", default_daylight_saving_end_month).unwrap();
    writeln!(f, "pub const DEFAULT_DAYLIGHT_SAVING_END_DAY: u8 = {};", default_daylight_saving_end_day).unwrap();
    writeln!(f, "pub const DEFAULT_DAYLIGHT_SAVING_END_HOUR: u8 = {};", default_daylight_saving_end_hour).unwrap();
    writeln!(f, "pub const DEFAULT_NTP_MSG_LEN: u16 = {};", default_ntp_msg_len).unwrap();
    writeln!(f, "pub const DEFAULT_NTP_PORT: u16 = {};", default_ntp_port).unwrap();
    writeln!(f, "pub const DEFAULT_NTP_SERVER: &str = {};", default_ntp_server).unwrap();
    writeln!(f, "pub const AES_KEY_SALT: &str = {};", hhg_aes_key_salt).unwrap();
    writeln!(f, "pub const AES_IV_SALT: &str = {};", hhg_aes_iv_salt).unwrap();
    writeln!(f, "pub const SYSTEM_USER_EMAIL: &str = {};", system_user_email).unwrap();
    writeln!(f, "pub const SYSTEM_USER_PASSWORD: &str = {};", system_user_password).unwrap();

    // Flush and close file explicitly
    f.flush().unwrap();
    drop(f);
    
    println!("cargo:warning=File written and flushed to: {}", dest_path.display());

    
    println!("cargo:rerun-if-changed={}", secrets_path().display());
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_WIFI_SSID");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_WIFI_PASSWORD");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_WIFI_AUTH");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_WIFI_ENABLED");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_TIMEZONE");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_DAYLIGHT_SAVING_ENABLED");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_MONTH");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_DAY");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_DAYLIGHT_SAVING_TIME_START_HOUR");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_MONTH");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_DAY");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_DAYLIGHT_SAVING_TIME_END_HOUR");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_NTP_MSG_LEN");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_NTP_PORT");
    println!("cargo:rerun-if-env-changed=HHG_DEFAULT_NTP_SERVER");
    println!("cargo:rerun-if-env-changed=HHG_AES_KEY_SALT");
    println!("cargo:rerun-if-env-changed=HHG_AES_IV_SALT");
    println!("cargo:rerun-if-env-changed=HHG_SYSTEM_USER_EMAIL");
    println!("cargo:rerun-if-env-changed=HHG_SYSTEM_USER_PASSWORD");
}
