pub(crate) mod deploy;
pub(crate) mod login;
pub(crate) mod logout;

use clap::Command;

pub fn command() -> Command {
  Command::new("cluster")
    .about("Cluster commands")
    .subcommand_required(true)
    .subcommand(deploy::command())
    .subcommand(login::command())
    .subcommand(logout::command())
}
