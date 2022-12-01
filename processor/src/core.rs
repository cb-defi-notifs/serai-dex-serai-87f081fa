/// The processor core module contains functionality that is shared across modules.
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::{env, fmt, thread, str::FromStr, io::Write};
use chrono::prelude::*;
use env_logger::fmt::Formatter;
use env_logger::Builder;
use log::info;
use log::{LevelFilter, Record};
use std::collections::HashMap;
// Key Generation
use message_box;
use std::alloc::System;
use zeroize::Zeroize;
use zalloc::ZeroizingAlloc;
use group::ff::PrimeField;
#[global_allocator]
static ZALLOC: ZeroizingAlloc<System> = ZeroizingAlloc(System);


#[derive(Clone, Debug, Deserialize)]
pub struct CoreProcess {
  core_config: CoreConfig,
}

impl CoreProcess {
  pub fn new(config: ProcessorConfig) -> Self {
    println!("New Core Process");
    let core_config = config.get_core();
    Self { core_config: core_config }
  }

  pub fn run(self) {
    println!("Starting Core Process");
    start_logger(true, String::from("core"));
  }

  fn stop(self) {
    println!("Stopping Core Process");
  }
}

fn start_logger(log_thread: bool, rust_log: String) {
  let output_format = move |formatter: &mut Formatter, record: &Record| {
    let thread_name = if log_thread {
      format!("(t: {}) ", thread::current().name().unwrap_or("unknown"))
    } else {
      "".to_string()
    };

    let local_time: DateTime<Local> = Local::now();
    let time_str = local_time.format("%H:%M:%S%.3f").to_string();
    write!(
      formatter,
      "{} {}{} - {} - {}\n",
      time_str,
      thread_name,
      record.level(),
      record.target(),
      record.args()
    )
  };
  let mut builder = Builder::new();
  builder.format(output_format).filter(None, LevelFilter::Info);

  builder.parse_filters(&rust_log);

  builder.init();
}

/// RunMode is Used to determine execution environment
#[derive(Copy, Deserialize)]
#[serde(tag = "type")]
pub enum RunMode {
  Development,
  Test,
  Production,
}

impl fmt::Display for RunMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RunMode::Development => write!(f, "development"),
      RunMode::Test => write!(f, "test"),
      RunMode::Production => write!(f, "production"),
    }
  }
}

impl std::str::FromStr for RunMode {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "development" => Ok(RunMode::Development),
      "test" => Ok(RunMode::Test),
      "production" => Ok(RunMode::Production),
      _ => Err(format!("{} is not a valid config option", s)),
    }
  }
}

impl fmt::Debug for RunMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RunMode::Development => write!(f, "development"),
      RunMode::Test => write!(f, "test"),
      RunMode::Production => write!(f, "production"),
    }
  }
}

impl Clone for RunMode {
  fn clone(&self) -> Self {
    match self {
      RunMode::Development => RunMode::Development,
      RunMode::Test => RunMode::Test,
      RunMode::Production => RunMode::Production,
    }
  }
}

#[derive(Copy, Deserialize)]
#[serde(tag = "type")]
pub enum ConfigType {
  Core,
  Chain,
  Health,
  Observer,
}

impl fmt::Display for ConfigType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ConfigType::Core => write!(f, "core"),
      ConfigType::Chain => write!(f, "chains"),
      ConfigType::Health => write!(f, "health"),
      ConfigType::Observer => write!(f, "observer"),
    }
  }
}

impl fmt::Debug for ConfigType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ConfigType::Core => write!(f, "core"),
      ConfigType::Chain => write!(f, "chains"),
      ConfigType::Health => write!(f, "health"),
      ConfigType::Observer => write!(f, "observer"),
    }
  }
}

impl Clone for ConfigType {
  fn clone(&self) -> Self {
    match self {
      ConfigType::Core => ConfigType::Core,
      ConfigType::Chain => ConfigType::Chain,
      ConfigType::Health => ConfigType::Health,
      ConfigType::Observer => ConfigType::Observer,
    }
  }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct CoreConfig {
  host: String,
  port: String,
}

impl CoreConfig {
  fn new(config: Config) -> Self {
    let host = config.get_string("host").unwrap();
    let port = config.get_string("port").unwrap();
    Self { host, port }
  }
  pub fn get_host(&self) -> String {
    self.host.clone()
  }
  pub fn get_port(&self) -> String {
    self.port.clone()
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct ChainConfig {
  btc: bool,
  eth: bool,
  xmr: bool,
}

impl ChainConfig {
  fn new(config: Config) -> Self {
    let btc = config.get_bool("chain_btc").unwrap();
    let eth = config.get_bool("chain_eth").unwrap();
    let xmr = config.get_bool("chain_xmr").unwrap();
    Self { btc, eth, xmr }
  }
  pub fn get_btc(&self) -> bool {
    self.btc
  }
  pub fn get_eth(&self) -> bool {
    self.eth
  }
  pub fn get_xmr(&self) -> bool {
    self.xmr
  }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
struct HealthConfig {}

impl HealthConfig {
  pub fn new() -> Self {
    Self {}
  }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
struct ObserverConfig {
  host: String,
  port: String,
  poll_interval: u16,
}

impl ObserverConfig {
  pub fn get_host(&self) -> String {
    self.host.clone()
  }

  pub fn get_port(&self) -> String {
    self.port.clone()
  }

  pub fn get_poll_interval(&self) -> u16 {
    self.poll_interval
  }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct ProcessorConfig {
  path: String,
  options: RunMode,
  core: CoreConfig,
  health: HealthConfig,
  observer: ObserverConfig,
  chain: ChainConfig,
}

impl ProcessorConfig {
  // Creates a new config based on the set environment
  pub fn new(path: String) -> Result<Self, ConfigError> {
    let run_mode = env::var("COORDINATOR_MODE").unwrap_or_else(|_| "development".into());

    let s = Config::builder()
      // Start off by merging in the "default" configuration file
      .add_source(File::with_name(&format!("{}/default", path)))
      // Add in the current environment file
      // Default to 'development' env
      // Note that this file is _optional_
      .add_source(File::with_name(&format!("{}/{}", path, run_mode)))
      .build()?;

    // Convert the config into enum for use.
    let mode = RunMode::from_str(&run_mode).unwrap_or(RunMode::Development);

    // switch based on mode to build config
    let config = ProcessorConfig {
      path: String::from("./config/"),
      options: mode.clone(),
      core: CoreConfig {
        port: s.get_string("core.port").unwrap(),
        host: s.get_string("core.host").unwrap(),
      },
      health: HealthConfig {},
      observer: ObserverConfig {
        host: String::from("localhost"),
        port: String::from("5050"),
        poll_interval: 1,
      },
      chain: ChainConfig{
        btc: s.get_bool("chains.btc").unwrap(),
        eth: s.get_bool("chains.eth").unwrap(),
        xmr: s.get_bool("chains.xmr").unwrap(),
      },
    };

    // Check coordinator pubkey env variable
    initialize_keys(&config);

    match mode {
      RunMode::Development => {
        // Set development specific config
        println!("Development config loaded");
        Ok(config)
      }
      RunMode::Test => {
        // Set test specific config
        println!("Test config loaded");
        Ok(config)
      }
      RunMode::Production => {
        // Set production specific config
        println!("Production config loaded");
        Ok(config)
      }
    }
  }

  // get the config path
  pub fn get_path(&self) -> String {
    self.path.clone()
  }
  // get the config options
  pub fn get_options(&self) -> RunMode {
    self.options.clone()
  }
  // get the core config
  pub fn get_core(&self) -> CoreConfig {
    self.core.clone()
  }
  // get the chain config
  pub fn get_chain(&self) -> ChainConfig {
    self.chain.clone()
  }
}

// Accepts startup argument specifying which coin package to start
pub fn initialize_coin(coin: &str) {
  println!("Received Coin Request: {}", coin);

  match coin {
    "btc" => {}
    "eth" => {}
    "xmr" => {}
    _ => println!("coin unavailable {}", coin),
  }
}

// Generates Private / Public key pair
pub fn initialize_keys(config: &ProcessorConfig) {
  let coin_hashmap = create_coin_hashmap(&config.chain);
  let hashmap_clone = coin_hashmap.clone();

  // Loop through each coin & if active, create pubkey consumer
  for (key, value) in hashmap_clone.into_iter() {
    if value == true {
      let mut env_privkey = String::from(&key).to_uppercase();
      env_privkey.push_str("_PRIV");

      // Checks if coin keys are set
      let priv_check = env::var(&env_privkey.to_string());
      if (priv_check.is_err()) {
        // Generates new private / public key
        let (privkey, pubkey) = message_box::key_gen();
        let mut privkey_bytes = unsafe { privkey.inner().to_repr() };
        // Sets private / public key to environment variables
        env::set_var(env_privkey, hex::encode(privkey_bytes.as_ref()));

        let mut env_pubkey = String::from(&key).to_uppercase();
        env_pubkey.push_str("_PUB");
        env::set_var(env_pubkey, hex::encode(pubkey.to_bytes()));
      }
    }
  }
}

// Create Hashmap based on coins
fn create_coin_hashmap(chain_config: &ChainConfig) -> HashMap<String, bool> {
  // Create Hashmap based on coins
  let j = serde_json::to_string(&chain_config).unwrap();
  let coins: HashMap<String, bool> = serde_json::from_str(&j).unwrap();
  coins
}