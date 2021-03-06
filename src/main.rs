#[macro_use]
mod macros;
mod args;
mod enricher;
mod exporter;
mod importer;
mod progressbar;
mod run;
use args::Args;

static mut VERBOSE: bool = false;

fn main() {
    match run() {
        Err(None) => std::process::exit(1),
        Err(Some(x)) => {
            eprintln!("{}", x);
            std::process::exit(1);
        }
        Ok(_) => std::process::exit(0),
    }
}

fn run() -> Result<(), Option<String>> {
    let args = Args::new();
    unsafe {
        VERBOSE = args.verbose;
    }
    args.validate()?;
    printlnv!("Args are {:?}.", args);
    run::run(args)
}
