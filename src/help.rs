use crate::{NAME, VERSION, DESCRIPTION};

pub fn print_help() {
    println!("{} {}", NAME, VERSION);
    println!();
    println!("{}", DESCRIPTION);
    println!();
}
