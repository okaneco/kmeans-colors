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
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    match opt.cmd {
        Some(Command::Find {
            input,
            colors,
            replace,
            max_iter,
            factor,
            runs,
            percentage,
            rgb,
            verbose,
            output,
            seed,
            transparent,
        }) => find_colors(
            input,
            colors,
            replace,
            max_iter,
            factor,
            runs,
            percentage,
            rgb,
            verbose,
            output,
            seed,
            transparent,
        )?,
        _ => run(opt)?,
    }

    Ok(())
}
