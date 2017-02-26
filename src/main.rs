extern crate nix;
extern crate clap;
mod tracee;
mod proot;
mod cli;

use proot::PRoot;
use tracee::FileSystemNameSpace;
use nix::unistd::getpid;

fn main() {
    // main memory of the program
    let mut proot: PRoot = PRoot::new();
    // memory for bindings and other struct
    let mut fs: FileSystemNameSpace = FileSystemNameSpace::new();

    // step 1: CLI parsing
    cli::get_config(&mut fs);

    // step 2: Pre-create the first tracee (pid == main pid)
    proot.create_tracee(getpid(), fs);

    // step 3: Start the first tracee
    proot.launch_process();

    if !proot.is_main_thread() {
        // For any tracee process we end the program here,
        // as what follows (event loop) is only for the main thread
        return;
    }

    // step 4: Configure the signal actions
    proot.prepare_sigactions();

    // step 5: Listen to and deal with tracees events
    proot.event_loop();

    println!("{:?}", proot);
}

