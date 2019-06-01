#![feature(slice_patterns)]

extern crate libc;
extern crate nix;
extern crate clap;
extern crate syscall;
#[macro_use]
extern crate lazy_static;
extern crate byteorder;

mod errors;
mod utils;
mod register;
mod kernel;
mod filesystem;
mod cli;
mod process;

use std::process::exit;
use process::sigactions;
use process::proot::{PRoot, stop_program, show_info};
use filesystem::{FileSystem, Initialiser};

fn main() {
    // step 1: CLI parsing
    let mut fs: FileSystem = FileSystem::new();

    cli::parse_config(&mut fs);

    if let Err(error) = fs.initialize() {
        eprintln!("Error during file system initialization: {}", error);
        exit(-1);
    }

    let mut proot: PRoot = PRoot::new();

    // step 2: Start the first tracee
    proot.launch_process(fs);

    // what follows (event loop) is only for the main thread,
    // as the child thread will stop after executing the `kernel.execve` command

    // step 3: Configure the signal actions
    sigactions::prepare_sigactions(stop_program, show_info);

    // step 4: Listen to and deal with tracees events
    proot.event_loop();

    println!("{:#?}", proot);
}
