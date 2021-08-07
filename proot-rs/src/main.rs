#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::redundant_field_names)]
#![feature(specialization)]

extern crate clap;
extern crate libc;
extern crate nix;
extern crate sc;
#[macro_use]
extern crate lazy_static;
extern crate byteorder;
#[macro_use]
extern crate log;

mod cli;
mod errors;
mod filesystem;
mod kernel;
mod process;
mod register;
mod utils;

use crate::errors::Result;
use crate::process::proot::{show_info, stop_program, PRoot};
use crate::process::sigactions;

fn run() -> Result<()> {
    // step 1: CLI parsing
    let (fs, command) = cli::parse_config()?;

    let mut proot: PRoot = PRoot::new();

    // step 2: initialize Proot and start the first tracee
    proot.init()?;
    proot.launch_process(fs, command)?;

    // what follows (event loop) is only for the main thread,
    // as the child thread will stop after executing the `kernel.execve` command

    // step 3: Configure the signal actions
    sigactions::prepare_sigactions(stop_program, show_info);

    // step 4: Listen to and deal with tracees events
    proot.event_loop()?;

    debug!(
        "first tracee exit with exit code: {}",
        proot.init_exit_code.unwrap()
    );

    std::process::exit(proot.init_exit_code.unwrap());
}

fn main() {
    env_logger::init();
    if let Err(err) = run() {
        error!("Exited with error: {}", err);
        std::process::exit(1);
    }
}
