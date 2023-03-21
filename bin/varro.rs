use clap::Parser;
use eyre::Result;

use varro::{
    client::Varro,
    config::Cli,
    telemetry,
};

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init(false)?;
    telemetry::register_shutdown();

    let cli = Cli::parse();
    let config = cli.to_config();
    match Varro::new(Some(config)).start().await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(target: "varro", "Varro exited with error: {}", e);
            Err(e)
        }
    }
}
