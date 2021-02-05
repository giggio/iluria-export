use clap::{App, AppSettings, Arg};

#[derive(Debug)]
pub struct Args {
    pub verbose: bool,
    pub file: String,
    pub url: String,
}

impl Args {
    pub fn new() -> Args {
        Args::new_from(&mut std::env::args_os()).unwrap_or_else(|err| err.exit())
    }
    fn new_from<I, T>(args: I) -> Result<Args, clap::Error>
    where
        I: Iterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let args = Args::get_args_app().get_matches_from_safe(args)?;
        Ok(Args::get_config_from_cl(args))
    }

    fn get_args_app<'a, 'b>() -> App<'a, 'b> {
        App::new("iluria-export")
            .version("0.1")
            .author("Giovanni Bassi <giggio@giggio.net>")
            .about("Export config from Iluria")
            .setting(AppSettings::ArgRequiredElseHelp)
            .arg(
                Arg::with_name("file")
                    .takes_value(true)
                    .index(1)
                    .required(true)
                    .help("File with products and variations")
                    .validator(|file| {
                        let path = std::path::Path::new(&file);
                        if path.exists() && path.is_file() {
                            Ok(())
                        } else {
                            Err("File does not exist".to_owned())
                        }
                    }),
            )
            .arg(
                Arg::with_name("url")
                    .takes_value(true)
                    .index(2)
                    .required(true)
                    .help("Base url to get products")
                    .validator(|supplied_url| {
                        let url_result = url::Url::parse(&supplied_url);
                        if let Ok(url) = url_result {
                            if url.cannot_be_a_base() {
                                return Err(format!("Url '{}' has to be absolute.", url));
                            }
                            if url.scheme() == "https" && url.scheme() == "http" {
                                return Err(format!(
                                    "Scheme '{}' has to be http or https.",
                                    url.scheme()
                                ));
                            } else {
                                Ok(())
                            }
                        } else {
                            Err("Invalid url format.".to_owned())
                        }
                    }),
            )
            .arg(
                Arg::with_name("v")
                    .short("v")
                    .long("verbose")
                    .global(true)
                    .multiple(true)
                    .help("Sets the level of verbosity"),
            )
    }

    fn get_config_from_cl(args: clap::ArgMatches) -> Args {
        let verbose = args.occurrences_of("v") > 0;
        let file = args
            .value_of("file")
            .expect("Should have file as it is required")
            .to_owned();
        let url = args
            .value_of("url")
            .expect("Should have url as it is required")
            .to_owned();
        Args { verbose, file, url }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    use pretty_assertions::assert_eq;

    #[test]
    fn args_show_expected_values() -> Result<(), String> {
        let file = std::env::current_exe()
            .map_err(|_| "Can't find exe.")?
            .to_str()
            .unwrap()
            .to_owned();
        let url = "http://foo";
        let args = Args::new_from(["iluria-export", &file, url, "--verbose"].iter())
            .map_err(|e| e.to_string())?;
        assert!(args.verbose);
        assert_eq!(file, args.file);
        assert_eq!(url, args.url);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn args_fail_when_file_doesnt_exist() {
        let file_path = std::env::temp_dir().join(format!("{}", rand::thread_rng().gen::<f64>()));
        let file = file_path.to_string_lossy();
        let url = "http://foo";
        Args::new_from(["iluria-export", &file, url, "--verbose"].iter())
            .map_err(|e| e.to_string())
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn args_fail_when_url_is_not_absolute() {
        let file = std::env::current_exe()
            .map_err(|_| "Can't find exe.")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let url = "mailto:x@sdjlkfsdljk.com";
        Args::new_from(["iluria-export", &file, url, "--verbose"].iter())
            .map_err(|e| e.to_string())
            .unwrap();
    }
}
