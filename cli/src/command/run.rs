use clap::{Arg, ArgAction, ArgMatches, Command};
use crate::config::Config;

pub fn command() -> Command {
  Command::new("run").about("Run a Dosei App")
    .arg(Arg::new("expose")
      .long("expose")
      .action(ArgAction::SetTrue)
      .help("Expose while running"))
}

pub fn run(arg_matches: &ArgMatches, config: &'static Config) -> anyhow::Result<()> {
  let exposed = arg_matches.get_one::<bool>("expose").unwrap();
  println!("{}", exposed);

  Ok(())
}
