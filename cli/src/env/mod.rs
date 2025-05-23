pub(crate) mod set;
pub(crate) mod unset;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
  /// Set environment variables
  Set {
    /// The env variable name
    #[arg(index = 1)]
    name: String,
    /// The env variable value
    #[arg(index = 2)]
    value: Option<String>,
  },
  /// Unset environment variables
  Unset {
    /// The env variable name
    #[arg(index = 1)]
    name: String,
  },
}
