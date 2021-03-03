use crate::titlemapper::TitleMapping;
use anyhow::{Context, Result};
use log::{error, info, warn};
use quick_xml::{events::Event, Reader};
use regex::bytes::Regex;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

pub fn gen_wikigraph<P: AsRef<Path>>(
    titlemap: &TitleMapping,
    datapath: P,
) -> Result<HashMap<u64, HashSet<u64>>> {
    lazy_static! {
        static ref LINK_RE: Regex =
            Regex::new(r"\[\x00*+\[(?:([^|#\]]*+)(?:#(?:[^|\]]*+))?)(?:\|(?:.*))?\]\x00*+\]")
                .expect("expected valid link extracting regex");
    }

    let mut edges: HashMap<u64, HashSet<u64>> = HashMap::new();
    let mut buf = Vec::new();

    let mut reader = Reader::from_file(datapath).context("expected valid file path")?;

    const LOG_MULT: u64 = 10_000;
    const EDGELOG_MULT: u64 = 100_000;
    let mut processed_pages = 0;
    let mut bad_edges = 0;
    let mut good_edges = 0;

    let mut page_id: Option<u64> = None;

    let mut in_page = false;
    let mut in_id = false;
    let mut in_text = false;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"page" => {
                    in_page = true;
                }
                b"id" => {
                    if in_page {
                        in_id = true;
                    } else {
                        warn!("<id> without parent <page>");
                    }
                }
                b"text" => {
                    if in_page {
                        in_text = true;
                    } else {
                        warn!("<text> without parent <page>");
                    }
                }
                _ => (),
            },
            Ok(Event::Text(ref e)) => {
                if in_id {
                    if page_id == None {
                        match std::str::from_utf8(e.escaped()) {
                            Ok(ref id_str) => match id_str.parse() {
                                Ok(id_num) => page_id = Some(id_num),
                                Err(e) => error!("could not parse id string to integer: {}", &e),
                            },
                            Err(e) => error!("could not parse id UTF-8 bytes: {}", &e),
                        }
                    }
                }
                if in_text {
                    if let Some(id) = page_id {
                        for caps in LINK_RE.captures_iter(e.escaped()) {
                            if let Some(bytes) = caps.get(1) {
                                match process_title(bytes.as_bytes()) {
                                    Ok(linkname) => match titlemap.str_u().get(&linkname) {
                                        Some(next_id) => {
                                            if let Some(set) = edges.get_mut(&id) {
                                                set.insert(*next_id);
                                            } else {
                                                edges.insert(id, HashSet::new());
                                                edges.get_mut(&id).unwrap().insert(*next_id);
                                            }
                                            good_edges += 1;
                                            if good_edges % EDGELOG_MULT == 0 {
                                                info!("{} good edges", good_edges);
                                            }
                                        }
                                        None => match titlemap.redirs().get(&linkname) {
                                            Some(next_str) => {
                                                match titlemap.str_u().get(next_str) {
                                                    Some(next_id) => {
                                                        if let Some(set) = edges.get_mut(&id) {
                                                            set.insert(*next_id);
                                                        } else {
                                                            edges.insert(id, HashSet::new());
                                                            edges
                                                                .get_mut(&id)
                                                                .unwrap()
                                                                .insert(*next_id);
                                                        }
                                                        good_edges += 1;
                                                        if good_edges % EDGELOG_MULT == 0 {
                                                            info!("{} good edges", good_edges);
                                                        }
                                                    }
                                                    None => {
                                                        bad_edges += 1;
                                                        if bad_edges % EDGELOG_MULT == 0 {
                                                            info!("{} bad edges", bad_edges);
                                                        }
                                                    }
                                                }
                                            }
                                            None => {
                                                bad_edges += 1;
                                                if bad_edges % EDGELOG_MULT == 0 {
                                                    warn!("{} bad edges", bad_edges);
                                                }
                                            }
                                        },
                                    },
                                    Err(e) => {
                                        error!("expected valid processing of title bytes: {}", e)
                                    }
                                }
                            } else {
                                warn!("found WikiLink with no title");
                            }
                        }
                    } else {
                        warn!("text with no page id");
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name() {
                b"page" => {
                    if !in_page {
                        warn!("closing </page> tag when not in <page>");
                    }
                    page_id = None;
                    in_text = false;
                    in_page = false;
                    processed_pages += 1;
                    if processed_pages % LOG_MULT == 0 {
                        info!("processed {} pages", processed_pages);
                    }
                }
                b"id" => {
                    if !in_id {
                        warn!("closing </id> tag when not in <id>");
                    }
                    in_id = false;
                }
                b"text" => {
                    if !in_text {
                        warn!("closing </text> tag when not in <text>");
                    }
                    in_text = false;
                }
                _ => (),
            },
            Ok(Event::Eof) => break,
            Err(ref e) => error!("error while reading XML file: {}", &e),
            _ => (),
        }
        buf.clear();
    }

    Ok(edges)
}

fn process_title(raw_title: &[u8]) -> Result<String> {
    lazy_static! {
        static ref SPACE_RE: Regex = Regex::new(r"[ _]+").expect("expected valid space regex");
        static ref TRIM_RE: Regex =
            Regex::new(r"^ ?(?P<title>[^ ].*[^ ]) $").expect("expect valid trim regex");
    }

    let mut title_str = TRIM_RE
        .replace(
            &SPACE_RE.replace_all(raw_title, &b" "[..]),
            &b"${title}"[..],
        )
        .into_owned();
    if !title_str.is_empty() && title_str[0] >= b'a' && title_str[0] <= b'z' {
        title_str[0] -= b'a' - b'A';
    }

    let title_str = String::from_utf8(title_str).context("expected valid UTF-8 bytes")?;
    Ok(title_str)
}
