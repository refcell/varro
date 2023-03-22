use std::{str::FromStr, time::Duration, fs::File, io::Write};

use clap::Parser;
use dirs::home_dir;
use ethers_core::types::Address;
use crate::{extract_env, config::Config, errors::ConfigError};

/// The [Varro] CLI
#[derive(Parser)]
pub struct Cli {
    /// The HTTP provider URL for L1.
    #[clap(short = 'l', long, env="L1_RPC_URL")]
    l1_client_rpc_url: Option<String>,
    /// The HTTP provider URL for the rollup node.
    #[clap(short = 'r', long, env="ROLLUP_NODE_RPC_URL")]
    rollup_node_rpc_url: Option<String>,
    /// The L2OutputOracle contract address.
    #[clap(short = 's', long, env="OUTPUT_ORACLE_ADDRESS")]
    output_oracle_address: Option<String>,
    /// The delay between querying L2 for more blocks and creating a new batch.
    #[clap(short = 'i', long, default_value = "2")]
    polling_interval: u64,
    /// The number of confirmations which to wait after appending new batches.
    #[clap(short = 'c', long, default_value = "2")]
    num_confirmation: u64,
    /// The number of [VarroError::NonceTooLow] errors required to give up on a
    /// tx at a particular nonce without receiving confirmation.
    #[clap(short = 'a', long, default_value = "2")]
    safe_abort_nonce_too_low: u64,
    /// The time to wait before resubmitting a transaction.
    #[clap(short = 'r', long, default_value = "2")]
    resubmission_timeout: u64,
    /// The HD seed used to derive the wallet private keys for both
    /// the sequencer and proposer. Must be used in conjunction with
    /// SequencerHDPath and ProposerHDPath.
    ///
    /// If not provided, a new mnemonic will be generated.
    #[clap(short = 'm', long, env="MNEMONIC")]
    mnemonic: Option<String>,
    /// The private key used for the L2 Output transactions.
    #[clap(short = 'k', long, env="OUTPUT_PRIVATE_KEY")]
    output_private_key: Option<String>,
    /// The derivation path used to obtain the private key for the output transactions.
    #[clap(short = 'p', long, default_value = "m/44'/60'/0'/0/0")]
    output_hd_path: String,
    /// Whether to use non-finalized L1 data to propose L2 blocks.
    #[clap(short = 'n', long)]
    allow_non_finalized: bool,
}

impl TryFrom<Cli> for Config {
    type Error = ConfigError;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        let l1_client_rpc_url = cli.l1_client_rpc_url.unwrap_or(extract_env!("L1_RPC_URL"));
        let rollup_node_rpc_url = cli.rollup_node_rpc_url.unwrap_or(extract_env!("ROLLUP_NODE_RPC_URL"));
        let output_oracle_address = cli.output_oracle_address.unwrap_or(extract_env!("OUTPUT_ORACLE_ADDRESS"));
        let polling_interval = Duration::from_secs(cli.polling_interval);
        let num_confirmation = cli.num_confirmation;
        let safe_abort_nonce_too_low = cli.safe_abort_nonce_too_low;
        let resubmission_timeout = Duration::from_secs(cli.resubmission_timeout);
        let mnemonic = cli.mnemonic.unwrap_or_else(|| extract_env!("MNEMONIC"));
        let output_private_key = cli.output_private_key.unwrap_or_else(|| extract_env!("OUTPUT_PRIVATE_KEY"));
        let output_hd_path = cli.output_hd_path;
        let allow_non_finalized = cli.allow_non_finalized;

        let config = Config {
            l1_client_rpc_url,
            rollup_node_rpc_url,
            output_oracle_address: Address::from_str(&output_oracle_address).map_err(|_| ConfigError::InvalidOutputOracleAddress(output_oracle_address))?,
            polling_interval,
            num_confirmation,
            safe_abort_nonce_too_low,
            resubmission_timeout,
            mnemonic,
            output_private_key,
            output_hd_path,
            allow_non_finalized,
        };

        // Save the config to disk
        let config_path = home_dir().unwrap().join(".varro/varro.toml");
        let mut file = File::create(&config_path).map_err(|_| ConfigError::TomlFileCreation(config_path.to_string_lossy().to_string()))?;
        file.write_all(toml::to_string(&config).map_err(|_| ConfigError::ConfigTomlConversion)?.as_bytes()).map_err(|_| ConfigError::TomlFileWrite)?;

        Ok(config)
    }
}
