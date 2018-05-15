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
extern crate toml;

mod parse;
mod index;
mod config;

use clap::{App, ArgMatches};
use parse::Parser;
use pbr::{ProgressBar, Units};
use index::{EventIndex, TopologyEvent};
use config::{ShowFormat, Config};
use std::io::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("check", Some(c_matches)) => check(c_matches),
        ("show", Some(s_matches)) => show(s_matches),
        _ => panic!("Unhandled subcommand!"),
    }
}

fn grab_config(matches: &ArgMatches) -> Config {
    let path = matches.value_of("config").unwrap_or("oida.toml");
    let mut file = File::open(&path).expect("Could not open config file!");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");
    toml::from_str(&contents).expect("could not parse config")
}

fn check(matches: &ArgMatches) {
    let config = grab_config(&matches);
    let check_config = config.check.unwrap();

    let pattern = check_config.pattern;
    let parser = Parser::new(&pattern).expect("Could not init parser");
    let mut index = EventIndex::new();
    
    let file = File::open(&check_config.input).expect("could not open file");
    let len = file.metadata().unwrap().len();
    let mut reader = BufReader::new(file);

    let mut pb = ProgressBar::new(len);
    pb.set_units(Units::Bytes);
    // Don't refresh more often so contention is on stdout which 
    // will actually slow the whole thing down...
    pb.set_max_refresh_rate(Some(Duration::from_millis(100)));
    pb.format("[-> ]");

    println!("> Starting to analyze \"{}\"", &check_config.input);
    let mut line = String::new();
    loop {
        let num_bytes = reader.read_line(&mut line);
        match num_bytes {
            Ok(num) if num > 0 => {
                pb.add(num as u64);
                let parsed = parser.parse(&line).expect(&format!("Could not parse line {:?}", line));
                if parsed.is_some() {
                    index.feed(parsed.unwrap());
                }
            },
            Ok(_) => break,
            Err(e) => panic!(format!("Error wile decoding line! {:?}", e)),
        }
        line.clear();
    }

    pb.finish();
    println!("> Completed");

    let output_path = check_config.output.unwrap_or("index.oida".into());
    println!("> Dumping Index into File \"{}\"", output_path);
    let mut buffer = File::create(&output_path).unwrap();
    index.serialize(&mut buffer);
    println!("> Completed")
}

fn show(matches: &ArgMatches) {
    let config = grab_config(&matches);
    let show_config = config.show.unwrap();

    let index_path = show_config.input.unwrap_or("index.oida".into());
    let input = File::open(&index_path).unwrap();

    println!("> Loading Index from File \"{}\"", &index_path);
    let index = EventIndex::from_reader(input);
    println!("> Completed");

    let format = show_config.format.unwrap_or(ShowFormat::Cli);
    match format {
        ShowFormat::Cli => show_cli(index),
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
