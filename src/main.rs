#![feature(io)]
#![feature(slice_patterns)]

extern crate libc;
extern crate nix;
extern crate clap;
extern crate syscall;
#[macro_use]
extern crate lazy_static;

mod errors;
mod utils;
mod register;
mod kernel;
mod filesystem;
mod cli;
mod process;

use process::sigactions;
use process::proot::{PRoot, stop_program, show_info};
use filesystem::fs::FileSystem;

fn main() {
    // step 1: CLI parsing
    let mut fs: FileSystem = FileSystem::new();
    cli::get_config(&mut fs);
    let mut proot: PRoot = PRoot::new(fs);

    // step 2: Start the first tracee
    proot.launch_process();

    // what follows (event loop) is only for the main thread,
    // as the child thread will stop after executing the `kernel.execve` command

    // step 3: Configure the signal actions
    sigactions::prepare_sigactions(stop_program, show_info);

    // step 4: Listen to and deal with tracees events
    proot.event_loop();

    println!("{:?}", proot);
}
