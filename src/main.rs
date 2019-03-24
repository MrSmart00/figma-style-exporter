#[macro_use]
extern crate tera;

mod figma;
mod design;

use std::collections::{HashSet, HashMap};
use std::{env, process};

use failure::Error;

use env_logger;
use log::info;

use figma::Node;
use design::Source;

const TEAM_ID_KEY: &str = "FIGMA_TEAM_ID";
const ACCESS_TOKEN_KEY: &str = "FIGMA_ACCESS_TOKEN";

fn main() -> Result<(), Error> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let access_token = match env::var(ACCESS_TOKEN_KEY) {
        Ok(val) => val,
        Err(err) => {
            println!("{}: {}", err, ACCESS_TOKEN_KEY);
            process::exit(1);
        },
    };

    let client = figma::Client::new(access_token);

    let team_id = match env::var(TEAM_ID_KEY) {
        Ok(val) => val,
        Err(err) => {
            println!("{}: {}", err, TEAM_ID_KEY);
            process::exit(1);
        },
    };

    let styles = r#try!(client.get_styles(&team_id));

    let file_keys = styles.iter().map(|style| {
        return style.file_key.clone();
    });

    let file_keys: HashSet<String> = file_keys.into_iter().collect();
    let mut file_nodes = HashMap::new();
    for file_key in file_keys {
    let file_styles: Vec<String> = styles.clone().into_iter().filter(|style| style.file_key == *file_key).map(|style| style.node_id).collect();
        file_nodes.insert(file_key, file_styles);
    }

    info!("{:#?}", file_nodes);

    let mut responses = Vec::new();
    for (file_key, file_node_ids) in file_nodes.iter() {
        let nodes = r#try!(client.get_file_nodes(&file_key, &file_node_ids));
        responses.push(nodes);
    }

    let nodes: Vec<Node> = responses.iter()
        .flat_map(|response| response)
        .cloned()
        .collect();

    let mut source = Source { rects: Vec::new(), texts: Vec::new() };
    for node in nodes {
        match node {
            Node::Rectangle { r } => {
                source.rects.push(r);
            },
            Node::Text { t } => {
                source.texts.push(t);
            },
        }
    }

    info!("{:#?}", source);

    source.generate();

    Ok(())
}
