use log::info;

fn main() {
    log4rs::init_file("./config/log4rs.yaml", Default::default())
        .expect("expected a log4rs config file");
    println!("Hello, world!");
    info!("started execution of wikigrapher");

    info!("completed execution of wikigrapher");
}
