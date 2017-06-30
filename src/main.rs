extern crate libc;
extern crate nix;
extern crate clap;
extern crate syscall;
mod constants;
mod utils;
#[macro_use]
mod regs;
mod syscalls;
mod bindings;
mod fsnamespace;
mod tracee;
mod cli;
mod sigactions;
mod proot;

use proot::{PRoot, stop_program, show_info};
use fsnamespace::FileSystemNamespace;

fn main() {
    // step 1: CLI parsing
    let mut fs: FileSystemNamespace = FileSystemNamespace::new();
    cli::get_config(&mut fs);
    let mut proot: PRoot = PRoot::new(fs);

    // step 2: Start the first tracee
    proot.launch_process();

    // what follows (event loop) is only for the main thread,
    // as the child thread will stop after executing the syscalls.execve command

    // step 3: Configure the signal actions
    sigactions::prepare_sigactions(stop_program, show_info);

    // step 4: Listen to and deal with tracees events
    proot.event_loop();

    println!("{:?}", proot);
}

