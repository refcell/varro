use clap::Parser;
use eyre::Result;

use varro::{
    builder::VarroBuilder,
    cli::Cli,
    config::Config,
    telemetry,
};

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init(false)?;
    telemetry::register_shutdown();

    let cli = Cli::parse();
    let config = Config::try_from(cli)?;
    let builder = VarroBuilder::try_from(config)?;
    let mut varro = builder.build()?;
    match varro.start().await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(target: "varro", "Varro exited with error: {}", e);
            Err(e)
        }
    }
}
