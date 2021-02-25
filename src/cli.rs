use clap::{App, Arg};
pub fn cli_app() -> App<'static, 'static> {
    App::new("WikiGrapher")
        .version("0.1.0")
        .author("Rishvic Pushpakaran <rishvic@gmail.com>")
        .about("Generates a graph of all the linked Wikipedia pages, can also process it")
        .arg(
            Arg::with_name("logconf")
                .short("l")
                .long("logconf")
                .value_name("LOGCONFIG")
                .help("Sets a custom log4rs configuration file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Sets the output file path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("DUMPFILE")
                .help("Path of the WikiDump file")
                .required(true)
                .index(1),
        )
}
