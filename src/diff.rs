use std::path::PathBuf;

#[derive(clap::Args)]
pub struct DiffArgs {
    left: PathBuf,
    right: PathBuf,
}

pub fn run(_args: DiffArgs) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
