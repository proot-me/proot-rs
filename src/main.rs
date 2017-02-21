extern crate nix;
extern crate clap;
mod tracee;
mod proot;
mod cli;

use proot::PRoot;

fn main() {
    cli::get_config();

    // main memory of the program
    let mut proot = PRoot::new();

    {
        // step 1: pre-create the first tracee (pid == 0)
        let tracee = proot.create_tracee(0);
        println!("{:?}", &tracee);
    }

    println!("{:?}", &proot);
}
