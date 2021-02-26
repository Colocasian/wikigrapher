use clap::{App, Arg, SubCommand};

pub fn cli_app() -> App<'static, 'static> {
    App::new("WikiGrapher")
        .version("0.1.1")
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
        .subcommand(
            SubCommand::with_name("genmap")
                .about("Generates title mapping from XML dump file")
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
                        .help("Path of the WikiDump XML file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("TMAPFILE")
                        .help("Path of the WikiDump TitleMap binary")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("gengraph")
                .about("Generates digraph from XML dump file and generated titlemap file")
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
                        .help("Path of the WikiDump XML file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("TMAPFILE")
                        .help("Path of the WikiDump TitleMap binary")
                        .required(true)
                        .index(2),
                ),
        )
}
