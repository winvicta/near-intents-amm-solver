//! Configuration read from environment variables.
//!
//! TS original: `src/configs/*.config.ts` and `src/configs/env.validation.ts`.
//! In TypeScript each setting is a module-level `const` read from
//! `process.env` when the file is imported. Here everything goes into one
//! `Config` struct that `main` builds once and passes around explicitly.
//!
//! TEE mode is out of scope for this port, so `TEE_ENABLED`,
//! `SOLVER_REGISTRY_CONTRACT` and `SOLVER_POOL_ID` are not read, and
//! `NEAR_ACCOUNT_ID` / `NEAR_PRIVATE_KEY` are always required (the TS
//! non-TEE rules).

use std::fmt;
use std::str::FromStr;

const DEFAULT_APP_PORT: u16 = 3000;
const DEFAULT_MARGIN_PERCENT: f64 = 0.3;
const DEFAULT_RELAY_WS_URL: &str = "wss://solver-relay-v2.chaindefuser.com/ws";
const DEFAULT_INTENTS_CONTRACT: &str = "intents.near";

// ---------------------------------------------------------------------------
// Small enums for settings that only have a few valid values.
// ---------------------------------------------------------------------------

/// Which NEAR network to talk to. TS: `NEAR_NETWORK_ID`, default mainnet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NearNetwork {
    Mainnet,
    Testnet,
}

impl NearNetwork {
    /// Default RPC nodes, same as `nearDefaultConnectionConfigs` in near.config.ts.
    pub fn default_node_urls(self) -> Vec<String> {
        let urls: &[&str] = match self {
            NearNetwork::Mainnet => &["https://free.rpc.fastnear.com", "https://near.lava.build"],
            NearNetwork::Testnet => &["https://test.rpc.fastnear.com", "https://neart.lava.build"],
        };
        urls.iter().map(|url| url.to_string()).collect()
    }
}

/// Parsing from a string goes through the standard `FromStr` trait, which is
/// what makes `"testnet".parse::<NearNetwork>()` work.
impl FromStr for NearNetwork {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(NearNetwork::Mainnet),
            "testnet" => Ok(NearNetwork::Testnet),
            _ => Err("expected mainnet or testnet".to_string()),
        }
    }
}

/// TS: `LOG_LEVEL`, one of error, warn, info, debug. Default info.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            _ => Err("expected error, warn, info or debug".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// A wrapper type so the private key never shows up in logs.
// ---------------------------------------------------------------------------

/// The solver's NEAR private key. A newtype around `String` with a
/// hand-written `Debug`, so `println!("{config:?}")` cannot leak it.
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateKey(String);

impl PrivateKey {
    /// The key itself. Only the signing code (a later step) should call this.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PrivateKey(<redacted>)")
    }
}

// ---------------------------------------------------------------------------
// The config struct and its error type.
// ---------------------------------------------------------------------------

/// All runtime settings of the solver.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// TS: `APP_PORT`, used by the health-check HTTP server.
    pub app_port: u16,
    pub log_level: LogLevel,
    pub network: NearNetwork,
    /// TS: `NEAR_NODE_URLS`, else `NEAR_NODE_URL`, else the network defaults.
    pub node_urls: Vec<String>,
    /// The two pool tokens as the relay names them: `nep141:<contract>`.
    pub tokens: [String; 2],
    pub relay_ws_url: String,
    pub relay_auth_key: Option<String>,
    pub intents_contract: String,
    /// Spread added on top of the pool price, in percent (0.3 means 0.3 %).
    pub margin_percent: f64,
    /// TS: `ONE_CLICK_API_ONLY=true` (read in quoter.service.ts).
    pub one_click_api_only: bool,
    pub account_id: String,
    pub private_key: PrivateKey,
}

/// Everything that can go wrong while reading the config.
/// A library returns a specific error enum so callers can `match` on it
/// (R4R p.58), rather than a string or an opaque error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A required variable is unset or empty.
    Missing { name: &'static str },
    /// A variable is set but its value is not acceptable.
    Invalid {
        name: &'static str,
        value: String,
        reason: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing { name } => write!(f, "{name} is required"),
            ConfigError::Invalid {
                name,
                value,
                reason,
            } => write!(f, "{name}={value:?} is invalid: {reason}"),
        }
    }
}

/// Implementing `std::error::Error` lets this type work with `?` and
/// `Box<dyn Error>` like any other error. `Debug + Display` is all it needs.
impl std::error::Error for ConfigError {}

impl Config {
    /// Read the config from the process environment.
    pub fn from_env() -> Result<Config, ConfigError> {
        Config::from_lookup(|name| std::env::var(name).ok())
    }

    /// Read the config through any lookup function. `from_env` passes one
    /// that reads real environment variables; tests pass one that reads a
    /// `HashMap`, so they never have to touch the process environment.
    pub fn from_lookup<F>(lookup: F) -> Result<Config, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        // TS uses `process.env.X || default`, so an empty string counts as
        // unset. This closure captures `lookup` and applies the same rule.
        let get = |name: &str| lookup(name).filter(|value| !value.trim().is_empty());

        // Parse an optional variable with `FromStr`, or fall back to a default.
        // A nested generic function: it is only visible inside `from_lookup`.
        fn parse_or<T: FromStr>(
            name: &'static str,
            raw: Option<String>,
            default: T,
        ) -> Result<T, ConfigError>
        where
            T::Err: fmt::Display,
        {
            match raw {
                None => Ok(default),
                Some(value) => value
                    .trim()
                    .parse()
                    .map_err(|err: T::Err| ConfigError::Invalid {
                        name,
                        reason: err.to_string(),
                        value,
                    }),
            }
        }

        let required = |name: &'static str| get(name).ok_or(ConfigError::Missing { name });

        let app_port = parse_or("APP_PORT", get("APP_PORT"), DEFAULT_APP_PORT)?;
        let log_level = parse_or("LOG_LEVEL", get("LOG_LEVEL"), LogLevel::Info)?;
        let network = parse_or(
            "NEAR_NETWORK_ID",
            get("NEAR_NETWORK_ID"),
            NearNetwork::Mainnet,
        )?;

        let node_urls = match get("NEAR_NODE_URLS").or_else(|| get("NEAR_NODE_URL")) {
            Some(list) => list.split(',').map(|url| url.trim().to_string()).collect(),
            None => network.default_node_urls(),
        };

        let token1 = required("AMM_TOKEN1_ID")?;
        let token2 = required("AMM_TOKEN2_ID")?;

        let margin_percent = parse_or(
            "MARGIN_PERCENT",
            get("MARGIN_PERCENT"),
            DEFAULT_MARGIN_PERCENT,
        )?;
        // Joi rule: `.positive()`. "NaN" parses as an f64, so reject it explicitly.
        if margin_percent.is_nan() || margin_percent <= 0.0 {
            return Err(ConfigError::Invalid {
                name: "MARGIN_PERCENT",
                value: margin_percent.to_string(),
                reason: "must be a positive number".to_string(),
            });
        }

        let account_id = required("NEAR_ACCOUNT_ID")?;
        let private_key = PrivateKey(required("NEAR_PRIVATE_KEY")?);

        Ok(Config {
            app_port,
            log_level,
            network,
            node_urls,
            tokens: [format!("nep141:{token1}"), format!("nep141:{token2}")],
            relay_ws_url: get("RELAY_WS_URL").unwrap_or_else(|| DEFAULT_RELAY_WS_URL.to_string()),
            relay_auth_key: get("RELAY_AUTH_KEY"),
            intents_contract: get("INTENTS_CONTRACT")
                .unwrap_or_else(|| DEFAULT_INTENTS_CONTRACT.to_string()),
            margin_percent,
            one_click_api_only: get("ONE_CLICK_API_ONLY")
                .is_some_and(|value| value.eq_ignore_ascii_case("true")),
            account_id,
            private_key,
        })
    }
}

// ---------------------------------------------------------------------------
// Unit tests. `#[cfg(test)]` means this module is compiled only by
// `cargo test`, never into the real binary (Rust 1 p.243, R4R p.88).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// The smallest set of variables that makes a valid config.
    fn minimal() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            ("AMM_TOKEN1_ID", "usdt.tether-token.near"),
            ("AMM_TOKEN2_ID", "wrap.near"),
            ("NEAR_ACCOUNT_ID", "solver1.near"),
            ("NEAR_PRIVATE_KEY", "ed25519:secretsecret"),
        ])
    }

    /// Build a config from a map instead of the real environment.
    fn load(vars: &HashMap<&'static str, &'static str>) -> Result<Config, ConfigError> {
        Config::from_lookup(|name| vars.get(name).map(|value| value.to_string()))
    }

    #[test]
    fn minimal_env_uses_ts_defaults() {
        let config = load(&minimal()).unwrap();

        assert_eq!(config.app_port, 3000);
        assert_eq!(config.log_level, LogLevel::Info);
        assert_eq!(config.network, NearNetwork::Mainnet);
        assert_eq!(
            config.node_urls,
            ["https://free.rpc.fastnear.com", "https://near.lava.build"]
        );
        assert_eq!(config.relay_ws_url, DEFAULT_RELAY_WS_URL);
        assert_eq!(config.relay_auth_key, None);
        assert_eq!(config.intents_contract, "intents.near");
        assert_eq!(config.margin_percent, 0.3);
        assert!(!config.one_click_api_only);
        assert_eq!(config.account_id, "solver1.near");
        assert_eq!(config.private_key.expose(), "ed25519:secretsecret");
    }

    #[test]
    fn tokens_get_nep141_prefix() {
        let config = load(&minimal()).unwrap();
        assert_eq!(
            config.tokens,
            ["nep141:usdt.tether-token.near", "nep141:wrap.near"]
        );
    }

    #[test]
    fn missing_required_variables_are_reported_by_name() {
        for name in [
            "AMM_TOKEN1_ID",
            "AMM_TOKEN2_ID",
            "NEAR_ACCOUNT_ID",
            "NEAR_PRIVATE_KEY",
        ] {
            let mut vars = minimal();
            vars.remove(name);
            assert_eq!(load(&vars), Err(ConfigError::Missing { name }));
        }
    }

    #[test]
    fn empty_string_counts_as_unset() {
        let mut vars = minimal();
        vars.insert("AMM_TOKEN1_ID", "   ");
        vars.insert("INTENTS_CONTRACT", "");
        assert_eq!(
            load(&vars),
            Err(ConfigError::Missing {
                name: "AMM_TOKEN1_ID"
            })
        );

        vars.insert("AMM_TOKEN1_ID", "usdt.tether-token.near");
        assert_eq!(load(&vars).unwrap().intents_contract, "intents.near");
    }

    #[test]
    fn testnet_switches_default_nodes() {
        let mut vars = minimal();
        vars.insert("NEAR_NETWORK_ID", "testnet");
        let config = load(&vars).unwrap();
        assert_eq!(config.network, NearNetwork::Testnet);
        assert_eq!(
            config.node_urls,
            ["https://test.rpc.fastnear.com", "https://neart.lava.build"]
        );
    }

    #[test]
    fn node_urls_list_is_split_trimmed_and_wins_over_single_url() {
        let mut vars = minimal();
        vars.insert("NEAR_NODE_URL", "https://single.example");
        vars.insert("NEAR_NODE_URLS", " https://a.example , https://b.example");
        assert_eq!(
            load(&vars).unwrap().node_urls,
            ["https://a.example", "https://b.example"]
        );

        vars.remove("NEAR_NODE_URLS");
        assert_eq!(load(&vars).unwrap().node_urls, ["https://single.example"]);
    }

    #[test]
    fn invalid_values_are_rejected() {
        let cases = [
            ("NEAR_NETWORK_ID", "betanet"),
            ("LOG_LEVEL", "verbose"),
            ("APP_PORT", "http"),
            ("APP_PORT", "70000"), // does not fit in a u16
            ("MARGIN_PERCENT", "abc"),
            ("MARGIN_PERCENT", "0"),
            ("MARGIN_PERCENT", "-1.5"),
            ("MARGIN_PERCENT", "NaN"),
        ];
        for (name, value) in cases {
            let mut vars = minimal();
            vars.insert(name, value);
            match load(&vars) {
                Err(ConfigError::Invalid { name: got, .. }) => assert_eq!(got, name),
                other => panic!("{name}={value}: expected Invalid, got {other:?}"),
            }
        }
    }

    #[test]
    fn optional_values_are_read() {
        let mut vars = minimal();
        vars.insert("APP_PORT", "8080");
        vars.insert("LOG_LEVEL", "debug");
        vars.insert("MARGIN_PERCENT", "1.25");
        vars.insert("RELAY_WS_URL", "wss://relay.example/ws");
        vars.insert("RELAY_AUTH_KEY", "key123");
        vars.insert("INTENTS_CONTRACT", "intents.testnet");
        vars.insert("ONE_CLICK_API_ONLY", "TRUE");

        let config = load(&vars).unwrap();
        assert_eq!(config.app_port, 8080);
        assert_eq!(config.log_level, LogLevel::Debug);
        assert_eq!(config.margin_percent, 1.25);
        assert_eq!(config.relay_ws_url, "wss://relay.example/ws");
        assert_eq!(config.relay_auth_key.as_deref(), Some("key123"));
        assert_eq!(config.intents_contract, "intents.testnet");
        assert!(config.one_click_api_only);
    }

    #[test]
    fn debug_output_hides_private_key() {
        let config = load(&minimal()).unwrap();
        let printed = format!("{config:?}");
        assert!(printed.contains("<redacted>"));
        assert!(!printed.contains("secretsecret"));
    }

    #[test]
    fn errors_have_readable_messages() {
        let missing = ConfigError::Missing {
            name: "AMM_TOKEN1_ID",
        };
        assert_eq!(missing.to_string(), "AMM_TOKEN1_ID is required");

        let mut vars = minimal();
        vars.insert("NEAR_NETWORK_ID", "betanet");
        assert_eq!(
            load(&vars).unwrap_err().to_string(),
            r#"NEAR_NETWORK_ID="betanet" is invalid: expected mainnet or testnet"#
        );
    }
}
