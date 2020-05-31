use crate::Result;

#[derive(Debug, PartialEq, Eq)]
pub enum Args {
    Help,
    Invocation(Invocation),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Invocation {
    pub command_name: String,
    pub verbose: bool,
    pub args: Vec<String>,
}

pub fn parse_args(args: &mut dyn Iterator<Item = String>) -> Result<Args> {
    if let Some(command) = args.next() {
        if &command == "--help" {
            return Ok(Args::Help);
        }
        let rest_args: Vec<_> = args.collect();
        return Ok(Args::Invocation(Invocation {
            command_name: command,
            verbose: false,
            args: rest_args,
        }));
    }
    Ok(Args::Help)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args(a: &[&str]) -> Vec<String> {
        a.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parse_no_args() {
        assert_eq!(parse_args(&mut vec![].into_iter()).unwrap(), Args::Help);
    }

    #[test]
    fn parse_help() {
        assert_eq!(
            parse_args(&mut make_args(&["--help"]).into_iter()).unwrap(),
            Args::Help
        );
    }

    #[test]
    fn parse_command() {
        assert_eq!(
            parse_args(&mut make_args(&["shake"]).into_iter()).unwrap(),
            Args::Invocation(Invocation {
                command_name: "shake".to_string(),
                verbose: false,
                args: vec![],
            })
        );
    }

    #[test]
    fn parse_command_with_args() {
        assert_eq!(
            parse_args(&mut make_args(&["stir", "--rotations", "42"]).into_iter()).unwrap(),
            Args::Invocation(Invocation {
                command_name: "stir".to_string(),
                verbose: false,
                args: make_args(&["--rotations", "42"]),
            })
        );
    }
}
