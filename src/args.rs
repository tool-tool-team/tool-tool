use crate::Result;

#[derive(Debug, PartialEq, Eq)]
pub enum Args {
    Help,
    Download,
    Invocation(Invocation),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Invocation {
    pub command_name: String,
    pub verbose: bool,
    pub from_shim: bool,
    pub args: Vec<String>,
}

pub fn parse_args(args: &mut dyn Iterator<Item = String>, verbose_env: bool) -> Result<Args> {
    if let Some(mut command) = args.next() {
        let mut verbose = verbose_env;
        let mut from_shim = false;
        if &command == "--help" {
            return Ok(Args::Help);
        }
        if &command == "--download" {
            return Ok(Args::Download);
        }
        let mut rest_args: Vec<_> = args.collect();
        loop {
            match command.as_str() {
                "-v" => {
                    if rest_args.is_empty() {
                        anyhow::bail!("tt: No command given")
                    }
                    verbose = true;
                    command = rest_args.remove(0);
                }
                "--from-shim" => {
                    if rest_args.is_empty() {
                        anyhow::bail!("tt: No command given")
                    }
                    from_shim = true;
                    command = rest_args.remove(0);
                }
                _ => break,
            }
        }
        return Ok(Args::Invocation(Invocation {
            command_name: command,
            verbose,
            from_shim,
            args: rest_args,
        }));
    }
    Ok(Args::Help)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_args(a: &[&str], verbose_env: bool) -> Args {
        parse_args(&mut a.iter().map(|x| x.to_string()), verbose_env).expect("Can be parsed")
    }

    fn make_args(a: &[&str]) -> Vec<String> {
        a.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parse_no_args() {
        assert_eq!(test_args(&[], false), Args::Help);
    }

    #[test]
    fn parse_help() {
        assert_eq!(test_args(&["--help"], false), Args::Help);
    }

    #[test]
    fn parse_download() {
        assert_eq!(test_args(&["--download"], false), Args::Download);
    }

    #[test]
    fn parse_command() {
        assert_eq!(
            test_args(&["shake"], false),
            Args::Invocation(Invocation {
                command_name: "shake".to_string(),
                verbose: false,
                from_shim: false,
                args: vec![],
            })
        );
    }

    #[test]
    fn parse_command_with_args() {
        assert_eq!(
            test_args(&["stir", "--rotations", "42"], false),
            Args::Invocation(Invocation {
                command_name: "stir".to_string(),
                verbose: false,
                from_shim: false,
                args: make_args(&["--rotations", "42"]),
            })
        );
    }

    #[test]
    fn parse_command_verbose_arg() {
        assert_eq!(
            test_args(&["-v", "foo", "bar"], false),
            Args::Invocation(Invocation {
                command_name: "foo".to_string(),
                verbose: true,
                from_shim: false,
                args: make_args(&["bar"]),
            })
        );
    }

    #[test]
    fn parse_command_verbose_env() {
        assert_eq!(
            test_args(&["foo", "bar"], true),
            Args::Invocation(Invocation {
                command_name: "foo".to_string(),
                verbose: true,
                from_shim: false,
                args: make_args(&["bar"]),
            })
        );
    }

    #[test]
    fn parse_command_from_shim() {
        assert_eq!(
            test_args(&["--from-shim", "foo", "bar"], false),
            Args::Invocation(Invocation {
                command_name: "foo".to_string(),
                verbose: false,
                from_shim: true,
                args: make_args(&["bar"]),
            })
        );
    }
}
