pub mod aoc1;
pub mod aoc2;
pub mod aoc3;
pub mod aoc4;
pub mod aoc5;
pub mod aoc6;
pub mod aoc7;
pub mod aoc8;
pub mod aoc9;
pub mod aoc10;
pub mod aoc11;
pub mod aoc12;
pub mod aoc13;
pub mod aoc14;
pub mod aoc15;
pub mod aoc16;
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
        .subcommand(SubCommand::with_name("aoc5")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc6")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc7")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc8")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc9")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc10")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc11")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc12")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc13")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc14")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc15")
                    .arg(Arg::with_name("part2")))
        .subcommand(SubCommand::with_name("aoc16")
                    .arg(Arg::with_name("part2")))
        .get_matches();
    match matches.subcommand() {
        ("aoc1", Some(sub_matches)) => aoc1::aoc1(sub_matches.is_present("part2"))?,
        ("aoc2", Some(sub_matches)) => aoc2::aoc2(sub_matches.is_present("part2"))?,
        ("aoc3", Some(sub_matches)) => aoc3::aoc3(sub_matches.is_present("part2"))?,
        ("aoc4", Some(sub_matches)) => aoc4::aoc4(sub_matches.is_present("part2"))?,
        ("aoc5", Some(sub_matches)) => aoc5::aoc5(sub_matches.is_present("part2"))?,
        ("aoc6", Some(sub_matches)) => aoc6::aoc6(sub_matches.is_present("part2"))?,
        ("aoc7", Some(sub_matches)) => aoc7::aoc7(sub_matches.is_present("part2"))?,
        ("aoc8", Some(sub_matches)) => aoc8::aoc8(sub_matches.is_present("part2"))?,
        ("aoc9", Some(sub_matches)) => aoc9::aoc9(sub_matches.is_present("part2"))?,
        ("aoc10", Some(sub_matches)) => aoc10::aoc10(sub_matches.is_present("part2"))?,
        ("aoc11", Some(sub_matches)) => aoc11::aoc11(sub_matches.is_present("part2"))?,
        ("aoc12", Some(sub_matches)) => aoc12::aoc12(sub_matches.is_present("part2"))?,
        ("aoc13", Some(sub_matches)) => aoc13::aoc13(sub_matches.is_present("part2"))?,
        ("aoc14", Some(sub_matches)) => aoc14::aoc14(sub_matches.is_present("part2"))?,
        ("aoc15", Some(sub_matches)) => aoc15::aoc15(sub_matches.is_present("part2"))?,
        ("aoc16", Some(sub_matches)) => aoc16::aoc16(sub_matches.is_present("part2"))?,
        _ => panic!("Invalid subcommand")
    }
    Ok(())
}
