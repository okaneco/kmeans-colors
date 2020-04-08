use std::error::Error;
use std::process;

use structopt::StructOpt;

mod lib;
use lib::{find_colors, run, Command, Opt};

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
        }) => find_colors(
            input, colors, replace, max_iter, factor, runs, percentage, rgb, verbose, output, seed,
        )?,
        _ => run(opt)?,
    }

    Ok(())
}
