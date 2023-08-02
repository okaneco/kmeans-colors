#![warn(rust_2018_idioms, unsafe_code)]

mod app;
mod args;
mod err;
mod filename;
mod utils;

use std::error::Error;
use std::process;

use structopt::StructOpt;

use app::{find_colors, run};
use args::{Command, Opt};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("kmeans_colors: {e}");
        process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    match opt.cmd {
        Some(command @ Command::Find { .. }) => find_colors(command)?,
        _ => run(opt)?,
    }

    Ok(())
}
