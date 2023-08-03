#![warn(rust_2018_idioms, unsafe_code)]
mod app;
mod args;
mod err;
mod filename;
mod find;
mod utils;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("kmeans_colors: {e}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let opt: args::Opt = structopt::StructOpt::from_args();
    match opt.cmd {
        Some(command @ args::Command::Find { .. }) => find::find_colors(command)?,
        _ => app::run(opt)?,
    }

    Ok(())
}
