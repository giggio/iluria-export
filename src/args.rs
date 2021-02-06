use std::path::Path;

use clap::{App, AppSettings, Arg};

#[derive(Debug)]
pub struct Args {
    pub verbose: bool,
    pub file: String,
    pub url: String,
    pub limit: u32,
    pub output_dir: Option<String>,
    pub output_products_file: String,
    pub output_variations_file: String,
    pub force: bool,
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
                            Err(format!("Input file '{}' does not exist", file))
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
                Arg::with_name("limit")
                    .short("l")
                    .long("limit")
                    .takes_value(true)
                    .required(false)
                    .help("How many item to process, by default all items will be processed")
                    .validator(|l| {
                        l.parse::<u32>()
                            .map(|_| ())
                            .map_err(|_| "Limit has to be an integer".to_owned())
                    }),
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .takes_value(true)
                    .required(false)
                    .help("Sets the output files directory, if not informed output will be printed to screen")
                    .validator(|dir| {
                        let path = std::path::Path::new(&dir);
                        if path.exists() && path.is_dir() {
                            Ok(())
                        } else {
                            Err(format!("Output directory '{}' does not exist", dir))
                        }
                    }),
            )
            .arg(
                Arg::with_name("products-file")
                    .short("p")
                    .long("products-file")
                    .takes_value(true)
                    .requires("output")
                    .help("Sets the output file name for the products file")
            )
            .arg(
                Arg::with_name("variations-file")
                    .short("r")
                    .long("variations-file")
                    .takes_value(true)
                    .requires("output")
                    .help("Sets the output file name for the variations file")
            )
            .arg(
                Arg::with_name("force")
                    .short("f")
                    .long("force")
                    .requires("output")
                    .help("Overwrite output files if they exist"),
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
        let limit = match args.value_of("limit") {
            Some(l) => l.parse::<u32>().expect("Limit should be a number."),
            None => 0,
        };
        Args {
            verbose,
            file,
            url,
            limit,
            output_dir: args.value_of("output").map(|s| s.to_owned()),
            output_products_file: args
                .value_of("products-file")
                .or(Some("products.csv"))
                .unwrap()
                .to_owned(),
            output_variations_file: args
                .value_of("variations-file")
                .or(Some("variations.csv"))
                .unwrap()
                .to_owned(),
            force: args.is_present("force"),
        }
    }

    pub fn get_output_files(&self) -> (Option<String>, Option<String>) {
        match &self.output_dir {
            None => (None, None),
            Some(output) => {
                let output_path = Path::new(output);
                (
                    Some(
                        output_path
                            .join(&self.output_products_file)
                            .to_string_lossy()
                            .as_ref()
                            .to_owned(),
                    ),
                    Some(
                        output_path
                            .join(&self.output_variations_file)
                            .to_string_lossy()
                            .as_ref()
                            .to_owned(),
                    ),
                )
            }
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let (products_file, variations_file) = self.get_output_files();
        if Args::file_exists(&products_file)? && !self.force {
            return Err(format!(
                "Output products file exists at '{}', use --force to overwrite.",
                products_file.unwrap()
            ));
        }
        if Args::file_exists(&variations_file)? && !self.force {
            return Err(format!(
                "Output variations file exists at '{}', use --force to overwrite.",
                variations_file.unwrap()
            ));
        }
        Ok(())
    }

    fn file_exists(file_option: &Option<String>) -> Result<bool, String> {
        match file_option {
            None => Ok(false),
            Some(file) => {
                let path = std::path::Path::new(&file);
                if path.exists() {
                    if path.is_file() {
                        Ok(true)
                    } else {
                        Err("Path is a directory".to_owned())
                    }
                } else {
                    Ok(false)
                }
            }
        }
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
