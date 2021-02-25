extern crate clap;

mod cli;

use anyhow::{Context, Result};
use log::info;
use std::fs::File;
use wikigrapher::titlemapper::TitleMapping;

fn main() -> Result<()> {
    let matches = cli::cli_app().get_matches();

    log4rs::init_file("./config/log4rs.yaml", Default::default())
        .expect("expected a log4rs config file");

    let dumpfile = &matches.args.get("DUMPFILE").unwrap().vals[0];
    let outfile = match matches.args.get("output") {
        Some(ref e) => e.vals[0].to_str().unwrap(),
        None => "out.cbor",
    };

    info!("started execution of wikigrapher");
    info!("processing dump file {:?}", dumpfile);

    let x = TitleMapping::from_file(dumpfile).context("titlemapper failed")?;

    let file = File::create(outfile).context("Could not create CBOR file")?;
    serde_cbor::to_writer(file, &x).context("Could not write to CBOR file")?;

    info!("completed execution of wikigrapher");
    Ok(())
}
