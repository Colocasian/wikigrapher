extern crate clap;
mod cli;

use anyhow::{Context, Result};
use log::{error, info};
use std::fs::File;
use wikigrapher::{grapher::gen_wikigraph, titlemapper::TitleMapping};

fn main() -> Result<()> {
    let matches = cli::cli_app().get_matches();

    let log4rs_conf = matches
        .value_of("logconf")
        .unwrap_or("./config/log4rs.yaml");

    log4rs::init_file(log4rs_conf, Default::default()).context("expected a log4rs config file")?;

    info!("started execution of wikigrapher");

    match matches.subcommand() {
        ("genmap", Some(sub_m)) => {
            let dumpfile = sub_m.value_of("DUMPFILE").unwrap();
            let outfile = sub_m.value_of("output").unwrap_or("titlemap.cbor");

            info!("started processing dumpfile");
            let titlemap = TitleMapping::from_file(dumpfile)
                .context("Could not process dumpfile to generate titlemapping")?;
            info!("finished processing dumpfile");

            info!("serializing TitleMapping to '{}'", outfile);
            serde_cbor::to_writer(
                File::create(outfile).context("Could not create output file")?,
                &titlemap,
            )
            .context("could not write to output file")?;
            info!("finished serializing TitleMapping");
        }
        ("gengraph", Some(sub_m)) => {
            let dumpfile = sub_m.value_of("DUMPFILE").unwrap();
            let tmapfile = sub_m.value_of("TMAPFILE").unwrap();
            let outfile = sub_m.value_of("output").unwrap_or("wikigraph.cbor");

            info!("started deserializing titlemap file");
            let titlemap: TitleMapping = serde_cbor::from_reader(
                File::open(tmapfile).context("could not open titlemap file")?,
            )
            .context("could not parse titlemap from file")?;
            info!("deserialized titlemap file!");

            info!("started processing dumpfile");
            let digraph = gen_wikigraph(&titlemap, dumpfile).context("could not generate graph")?;
            info!("completed graph generation");

            info!("serializing digraph to '{}'", outfile);
            serde_cbor::to_writer(
                File::create(outfile).context("Could not create output file")?,
                &digraph,
            )
            .context("could not write to output file")?;
            info!("finished serializing digraph");
        }
        _ => error!("one of the valid subcommands are required, exiting..."),
    }

    info!("completed execution of wikigrapher");
    Ok(())
}
