# Iluria export

This project exports a file from the Iluria e-commerce site to make it available
to be imported into Tray.

## Quick Start

Download it from the releases (when there is one) or build it with `cargo build`.

Run it like this:

````bash
iluria-export /path/to/iluria-relatorio-de-estoque-dos-produtos.csv https://your_store_at_iluria.com.br -o /path/to/export/
````

After running will have two .csv files at the exported locations, one for the
products, another for the variations.

### Detailed options

Run `iluria-export --help` to see the options. When this readme was written the
options were these:

````text
iluria-export 0.1
Giovanni Bassi <giggio@giggio.net>
Export config from Iluria

USAGE:
    iluria-export [FLAGS] [OPTIONS] <file> <url>

FLAGS:
    -f, --force       Overwrite output files if they exist
    -h, --help        Prints help information
    -s, --simulate    Simulate calls to scraping endpoints
    -v, --verbose     Sets the level of verbosity
    -V, --version     Prints version information

OPTIONS:
    -l, --limit <limit>                        How many item to process, by default all items will be processed
    -o, --output <output>                      Sets the output files directory, if not informed output will be printed
                                               to screen
    -p, --products-file <products-file>        Sets the output file name for the products file
    -r, --variations-file <variations-file>    Sets the output file name for the variations file

ARGS:
    <file>    File with products and variations
    <url>     Base url to get products
````

## Contributing

Questions, comments, bug reports, and pull requests are all welcome.  Submit them at
[the project on GitHub](https://github.com/giggio/iluria-export/).

Bug reports that include steps-to-reproduce (including code) are the
best. Even better, make them in the form of pull requests.

## Author

[Giovanni Bassi](https://github.com/giggio)

## License

Licensed under the MIT license.
