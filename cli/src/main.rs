mod app;
pub(crate) mod auth;
mod cluster;
mod config;
pub(crate) mod deploy;
mod env;
mod file;
pub(crate) mod init;
pub(crate) mod run;
pub(crate) mod whoami;

use crate::cluster::command::login::login;
use crate::cluster::command::logout::logout;
use crate::config::Config;
use clap::Command;
use deploy::deploy;
use init::init;
use run::run;
use whoami::whoami;

fn cli() -> Command {
  Command::new("dosei")
    .version(env!("CARGO_PKG_VERSION"))
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(init::command())
    .subcommand(run::command())
    .subcommand(whoami::command())
    .subcommand(deploy::command())
    .subcommand(cluster::command::command())
    .subcommand(env::command())
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  match cli().get_matches().subcommand() {
    Some(("run", arg_matches)) => run(arg_matches, config)?,
    Some(("init", arg_matches)) => init(arg_matches)?,
    Some(("whoami", _)) => whoami(config)?,
    Some(("env", params)) => match params.subcommand() {
      Some(("set", arg_matches)) => env::set::set_env(arg_matches, config)?,
      Some(("unset", arg_matches)) => env::unset::unset_env(arg_matches, config)?,
      _ => unreachable!(),
    },
    Some(("cluster", params)) => match params.subcommand() {
      Some(("login", arg_matches)) => login(arg_matches, config)?,
      Some(("logout", _)) => logout(config)?,
      Some(("deploy", arg_matches)) => cluster::command::deploy::deploy(arg_matches, config)?,
      _ => unreachable!(),
    },
    Some(("deploy", _)) => deploy(config)?,
    _ => unreachable!(),
  };
  Ok(())
}
