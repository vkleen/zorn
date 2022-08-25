use tracing::debug;
use anyhow::Result;

#[derive(Debug, clap::Args)]
pub(super) struct CmdArgs {
}

impl CmdArgs {
    #[tracing::instrument]
    pub(super) fn run(&self) -> Result<()> {
        debug!("Encryption placeholder");
        Ok(())
    }
}

pub(super) const ABOUT: &str = indoc::indoc!{"
    Encrypt a message
"};
