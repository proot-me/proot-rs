extern crate nix;
mod tracee;
mod proot;
use tracee::Tracee;
use proot::PRoot;

fn main() {
    // main memory of the program
    let mut proot = PRoot::new();
    {
        // step 1: pre-create the first tracee (pid == 0)
        let tracee1 = proot.create_tracee(0);
    }
    println!("{:?}", &proot);
}
