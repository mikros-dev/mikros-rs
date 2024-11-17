
#[derive(Debug)]
pub(crate) struct Args {
    pub config_path: Option<String>,
    help: bool,
    service_name: String,
}

impl Args {
    pub fn load() -> Self {
        let args: Vec<String> = std::env::args().collect();
        match Self::parse(&args) {
            Ok(args) => {
                if args.help {
                    Self::usage(&args.service_name);
                    std::process::exit(0);
                }

                args
            },
            Err(msg) => {
                eprintln!("{}", msg);
                std::process::exit(1);
            }
        }
    }

    fn parse(args: &[String]) -> Result<Self, String> {
        let mut config = Args {
            help: false,
            config_path: None,
            service_name: args[0].clone(),
        };

        let mut iter = args.iter().enumerate().peekable();

        // Skips the service name
        iter.next();

        // Parse other arguments
        while let Some((_, arg)) = iter.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    config.help = true;
                }
                "--config" => {
                    match iter.peek() {
                        None => return Err("error: --config option requires a file path".to_string()),
                        Some((_, next_arg)) => {
                            config.config_path = Some(next_arg.to_string());
                            iter.next();
                        }
                    }
                }
                _ => {
                    return Err(format!("unknown argument: {}", arg));
                }
            }
        }

        Ok(config)
    }

    fn usage(service_name: &str) {
        println!("Usage: {} [OPTIONS]", service_name);
        println!();
        println!("Options:");
        println!("  -h, --help      Print this help menu.");
        println!("  --config <path> Specify an alternative 'service.toml' config file.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_args() {
        let args = vec!["service".to_string()];
        let result = Args::parse(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().help, false);
    }

    #[test]
    fn test_help() {
        let args = vec!["service".to_string(), "--help".to_string()];
        let result = Args::parse(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().help, true);
    }

    #[test]
    fn test_config_option() {
        let args = vec!["service".to_string(), "--config".to_string(), "/path/to/service.toml".to_string()];
        let result = Args::parse(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().config_path.unwrap(), "/path/to/service.toml");
    }

    #[test]
    fn test_missing_config_path_option() {
        let args = vec!["service".to_string(), "--config".to_string()];
        let result = Args::parse(&args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "error: --config option requires a file path");
    }

    #[test]
    fn test_unknown_option() {
        let args = vec!["service".to_string(), "--unknown".to_string()];
        let result = Args::parse(&args);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "unknown argument: --unknown");
    }
}