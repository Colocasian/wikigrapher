use anyhow::{Context, Result};
use log::{error, info, warn};
use quick_xml::{events::Event, Reader};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Serialize, Deserialize)]
pub struct TitleMapping {
    str_u: HashMap<String, u64>,
    u_str: HashMap<u64, String>,
    redirs: HashMap<String, String>,
}

impl TitleMapping {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<TitleMapping> {
        let mut reader = Reader::from_file(path).with_context(|| {
            error!("could not read XML file");
            "Failed to read XML file"
        })?;
        let mut buf = Vec::new();

        let mut titlemap = TitleMapping {
            str_u: HashMap::new(),
            u_str: HashMap::new(),
            redirs: HashMap::new(),
        };

        const LOG_MULT: u64 = 10_000;
        let mut processed_pages = 0;

        let mut page_title = None;
        let mut page_redirect = None;
        let mut page_id: Option<u64> = None;

        let mut in_page = false;
        let mut in_title = false;
        let mut in_id = false;

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"page" => {
                        in_page = true;
                    }
                    b"title" => {
                        if in_page {
                            in_title = true;
                        } else {
                            warn!("<title> without parent <page>");
                        }
                    }
                    b"id" => {
                        if in_page {
                            in_id = true;
                        } else {
                            warn!("<id> without parent <page>");
                        }
                    }
                    _ => (),
                },
                Ok(Event::Empty(ref e)) => {
                    if e.name() == b"redirect" {
                        for attribute in e.attributes() {
                            match attribute {
                                Ok(attr) => {
                                    if attr.key == b"title" {
                                        page_redirect = Some(attr.value.into_owned());
                                    }
                                }
                                Err(e) => warn!("malformed <redirect /> attribute: {}", e),
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_title {
                        page_title = Some(e.escaped().to_vec());
                    }
                    if in_id {
                        if page_id == None {
                            match std::str::from_utf8(e.escaped()) {
                                Ok(ref id_str) => match id_str.parse() {
                                    Ok(id_num) => page_id = Some(id_num),
                                    Err(e) => {
                                        error!("could not parse id string to integer: {}", &e)
                                    }
                                },
                                Err(e) => error!("could not parse id UTF-8 bytes: {}", &e),
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => match e.name() {
                    b"page" => {
                        if !in_page {
                            warn!("closing </page> tag when not in <page>");
                        }
                        if let Some(title) = page_title {
                            if let Some(redir) = page_redirect {
                                titlemap.redirs.insert(
                                    String::from_utf8(title).unwrap(),
                                    String::from_utf8(redir).unwrap(),
                                );
                            } else {
                                if let Some(id) = page_id {
                                    titlemap
                                        .u_str
                                        .insert(id, String::from_utf8(title.clone()).unwrap());
                                    titlemap.str_u.insert(String::from_utf8(title).unwrap(), id);
                                }
                            }
                        }
                        page_title = None;
                        page_redirect = None;
                        page_id = None;
                        in_page = false;
                        processed_pages += 1;
                        if processed_pages % LOG_MULT == 0 {
                            info!("processed {} pages", processed_pages);
                        }
                    }
                    b"title" => {
                        if !in_title {
                            warn!("closing </title> tag when not in <title>");
                        }
                        in_title = false;
                    }
                    b"id" => {
                        if !in_id {
                            warn!("closing </id> tag when not in <id>");
                        }
                        in_id = false;
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break,
                Err(ref e) => error!("error while reading XML file: {}", &e),
                _ => (),
            }
            buf.clear();
        }

        info!("processed a total of {} pages", processed_pages);
        Ok(titlemap)
    }

    pub fn str_u(&self) -> &HashMap<String, u64> {
        &self.str_u
    }
    pub fn u_str(&self) -> &HashMap<u64, String> {
        &self.u_str
    }
    pub fn redirs(&self) -> &HashMap<String, String> {
        &self.redirs
    }
}
