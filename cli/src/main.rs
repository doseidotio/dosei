mod cli;
mod cluster;
mod config;
mod deploy;
mod env;
mod file;
mod init;
mod run;
mod ssh;
mod table;

use crate::cli::{Cli, Commands};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::io;
use std::io::Write;

fn main() -> anyhow::Result<()> {
  match Cli::parse().command {
    Commands::Init { path, template } => init::command(path, template)?,
    // Commands::Run { expose } => run::command(expose)?,
    Commands::Deploy {
      cluster_name,
      allow_dirty,
    } => {
      Cli::check_allow_dirty(allow_dirty)?;
      deploy::command(cluster_name, allow_dirty)?
    }
    Commands::Cluster { command } => match command {
      cluster::command::Commands::Connect => cluster::command::connect::command()?,
      cluster::command::Commands::Deploy {
        allow_dirty,
        allow_invalid_domain,
      } => {
        Cli::check_allow_dirty(allow_dirty)?;
        cluster::command::deploy::command(allow_invalid_domain)?
      }
      cluster::command::Commands::Login {
        name,
        username,
        yes,
      } => cluster::command::login::command(name, username, yes)?,
      cluster::command::Commands::Dashboard { name } => cluster::command::dashboard::command(name)?,
      cluster::command::Commands::Logs => cluster::command::logs::command()?,
      cluster::command::Commands::Ls => cluster::command::ls::command()?,
      cluster::command::Commands::SetDefault { name } => {
        cluster::command::set_default::command(name)?
      }
      cluster::command::Commands::Default => cluster::command::default::command()?,
    },
    // Commands::Env { command } => match command {
    //   env::Commands::Set { name, value } => env::set::command(name, value, config)?,
    //   env::Commands::Unset { name } => env::unset::command(name, config)?,
    // },
    Commands::Completion { shell } => {
      let mut cmd = Cli::command();
      let name = cmd.get_name().to_string();

      let mut stdout = io::stdout();
      generate(shell, &mut cmd, name, &mut stdout);
      stdout.flush()?;
    }
  }
  Ok(())
}
