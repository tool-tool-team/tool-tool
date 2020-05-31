use crate::{DESCRIPTION, NAME, VERSION};

pub fn print_help() {
    println!("{} {}", NAME, VERSION);
    println!();
    println!("{}", DESCRIPTION);
    println!();
}
