pub mod aoc1;
pub mod aoc2;
pub mod aoc3;
pub mod aoc4;
use clap::{Arg, App, SubCommand};
use failure::Error;

fn main() -> Result<(), Error> {
    let matches = App::new("Advent of Code 2018")
        .subcommand(SubCommand::with_name("aoc1")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc2")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc3")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc4")
                    .arg(Arg::with_name("part2")))
        .get_matches();
    match matches.subcommand() {
        ("aoc1", Some(sub_matches)) => aoc1::aoc1(sub_matches.is_present("part2"))?,
        ("aoc2", Some(sub_matches)) => aoc2::aoc2(sub_matches.is_present("part2"))?,
        ("aoc3", Some(sub_matches)) => aoc3::aoc3(sub_matches.is_present("part2"))?,
        ("aoc4", Some(sub_matches)) => aoc4::aoc4(sub_matches.is_present("part2"))?,
        _ => panic!("Invalid subcommand")
    }
    Ok(())
}
