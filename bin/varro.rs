use clap::Parser;
use eyre::Result;

use varro::{
    cli::Cli,
    telemetry,
    config::Config,
    builder::VarroBuilder,
};

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init(false)?;
    telemetry::register_shutdown();

    let cli = Cli::parse();
    let config =  Config::try_from(cli)?;
    let builder = VarroBuilder::try_from(config)?;
    let varro = builder.build()?;
    match varro.start().await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(target: "varro", "Varro exited with error: {}", e);
            Err(e)
        }
    }
}
