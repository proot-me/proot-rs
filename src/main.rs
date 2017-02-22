extern crate nix;
extern crate clap;
mod tracee;
mod proot;
mod cli;

use proot::PRoot;
use tracee::FileSystemNameSpace;

fn main() {
    // main memory of the program
    let mut proot: PRoot = PRoot::new();
    // memory for bindings and other struct
    let mut fs: FileSystemNameSpace = FileSystemNameSpace::new();

    // step 1: CLI parsing
    cli::get_config(&mut fs);
    println!("{:?}", fs);

    {
        // step 2: Pre-create the first tracee (pid == 0)
        let tracee = proot.create_tracee(0, fs);
        println!("{:?}", &tracee);
    }


    println!("{:?}", proot);
}
