extern crate chrono;
extern crate grok;
extern crate pbr;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate colored;
extern crate serde_cbor;

mod source;
mod parse;
mod index;

use clap::{App, ArgMatches};
use source::{FileSource, Source};
use parse::Parser;
use index::EventIndex;
use pbr::{ProgressBar, Units};
use std::fs::File;
use index::TopologyEvent;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("check", Some(c_matches)) => check(c_matches),
        ("show", Some(s_matches)) => show(s_matches),
        _ => panic!("Unhandled subcommand!"),
    }
}

fn check(matches: &ArgMatches) {
    let filepath = matches
        .value_of("input")
        .expect("filepath not found (input)");

    let source = FileSource::new(filepath).expect("Could not init file source");
    let parser = Parser::new().expect("Could not init parser");
    let mut index = EventIndex::new();

    let total_file_size = source.size();
    let mut pb = ProgressBar::new(total_file_size);
    pb.set_units(Units::Bytes);
    pb.format("[-> ]");

    println!("> Starting to analyze \"{}\"", filepath);

    for log_line in source
        .filter_map(|l| Some(l.expect("Could not decode line!")))
        .filter_map(|l| {
            pb.add(l.len() as u64);
            parser
                .parse(&l)
                .expect(&format!("Could not parse line {:?}", l))
        }) {
        index.feed(log_line);
    }

    pb.finish();

    println!("> Completed");

    let output_path = matches.value_of("output").unwrap_or("index.oida");
    println!("> Dumping Index into File \"{}\"", output_path);
    let mut buffer = File::create(&output_path).unwrap();
    index.serialize(&mut buffer);
    println!("> Completed")
}

fn show(matches: &ArgMatches) {
    let index_path = matches.value_of("input").unwrap_or("index.oida");
    let input = File::open(index_path).unwrap();

    println!("> Loading Index from File \"{}\"", index_path);
    let index = EventIndex::from_reader(input);
    println!("> Completed");

    let format = matches.value_of("format").unwrap_or("cli");
    match format {
        "cli" => show_cli(index),
        _ => panic!("Unsupported format"),
    }
}

fn show_cli(index: EventIndex) {
    use colored::*;

    println!("> Printing Stats to CLI\n");

    println!("Topology Events");
    println!("---------------\n");

    for ev in index.topo_events {
        match ev {
            TopologyEvent::ConnectingNode(d, n) => println!(
                "  {} {} {}",
                d.format("%H:%M:%S").to_string(),
                "node +".green(),
                n
            ),
            TopologyEvent::DisconnectingNode(d, n) => println!(
                "  {} {} {}",
                d.format("%H:%M:%S").to_string(),
                "node -".red(),
                n
            ),
        }
    }
}
