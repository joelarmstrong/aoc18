extern crate clap;
#[macro_use]
extern crate failure;

pub mod aoc1;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("Advent of Code 2018")
        .subcommand(SubCommand::with_name("aoc1")
                    .arg(Arg::with_name("part2")))
        .get_matches();
    match matches.subcommand() {
        ("aoc1", Some(sub_matches)) => aoc1::aoc1(sub_matches.is_present("part2")),
        _ => panic!("Invalid subcommand")
    }
}
