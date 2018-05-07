extern crate chrono;
extern crate grok;
extern crate regex;
extern crate pbr;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

mod source;
mod parse;
mod index;

use clap::{ArgMatches, App};
use source::{Source, FileSource};
use parse::Parser;
use index::EventIndex;
use pbr::{Units, ProgressBar};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand() {
        ("check", Some(c_matches)) => check(c_matches),
        _ => panic!("Unhandled subcommand!"),
    }
}

fn check(matches: &ArgMatches) {
    let filepath = matches.value_of("input").expect("filepath not found (input)");

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

    println!("{:#?}", index);
}