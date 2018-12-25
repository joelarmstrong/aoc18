use std::collections::HashSet;
use std::cmp::min;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::io::BufRead;
use std::ops::RangeInclusive;
use failure::{Error, bail, format_err};
use regex::Regex;

pub fn aoc18(part2: bool) -> Result<(), Error> {
    let mut lumber = parse_lumber(&mut io::stdin().lock())?;
    if part2 {
    } else {
        lumber.advance_multiple(10);
        println!("Resource value: {}", lumber.resource_value());
    }
    Ok(())
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum AcreContents {
    Open,
    Trees,
    Lumberyard,
}

use self::AcreContents::*;

struct LumberCollection {
    grid: Vec<Vec<AcreContents>>,
}

impl LumberCollection {
    fn advance(&mut self) {
        let mut new_grid = vec![];
        for y in 0..self.grid.len() {
            let mut new_row = vec![];
            for x in 0..self.grid[y].len() {
                let neighbors = self.neighbors(x, y);
                let new_char = match self.grid[y][x] {
                    Open => {
                        if neighbors.iter().filter(|&&n| n == Trees).count() >= 3 {
                            Trees
                        } else {
                            Open
                        }
                    },
                    Trees => {
                        if neighbors.iter().filter(|&&n| n == Lumberyard).count() >= 3 {
                            Lumberyard
                        } else {
                            Trees
                        }
                    },
                    Lumberyard => {
                        let lumberyards = neighbors.iter().filter(|&&n| n == Lumberyard).count();
                        let trees = neighbors.iter().filter(|&&n| n == Trees).count();
                        if trees >= 1 && lumberyards >= 1 {
                            Lumberyard
                        } else {
                            Open
                        }
                    },
                };
                new_row.push(new_char);
            }
            new_grid.push(new_row);
        }
        self.grid = new_grid;
    }

    fn advance_multiple(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn neighbors(&self, x: usize, y: usize) -> Vec<AcreContents> {
        let possible_adjacencies = vec![
            (x.checked_sub(1), y.checked_sub(1)),
            (Some(x), y.checked_sub(1)),
            (Some(x + 1), y.checked_sub(1)),
            (x.checked_sub(1), Some(y)),
            (Some(x + 1), Some(y)),
            (x.checked_sub(1), Some(y + 1)),
            (Some(x), Some(y + 1)),
            (Some(x + 1), Some(y + 1)),
        ];
        let mut neighbors = vec![];
        for possible_adjacency in possible_adjacencies {
            match possible_adjacency {
                (Some(x), Some(y)) => {
                    if y < self.grid.len() && x < self.grid[y].len() {
                        neighbors.push(self.grid[y][x]);
                    }
                },
                _ => {},
            }
        }
        neighbors
    }

    fn resource_value(&self) -> usize {
        let lumberyards: usize = self.grid.iter().map(|r| r.iter().filter(|&&a| a == Lumberyard).count()).sum();
        let trees: usize = self.grid.iter().map(|r| r.iter().filter(|&&a| a == Trees).count()).sum();
        lumberyards * trees
    }
}

impl Display for LumberCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.grid {
            for entry in row {
                let ch = match entry {
                    Open => '.',
                    Trees => '|',
                    Lumberyard => '#',
                };
                write!(f, "{}", ch)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn parse_lumber(input: &mut impl BufRead) -> Result<LumberCollection, Error> {
    let mut grid = vec![];
    for line_res in input.lines() {
        let line = line_res?;
        let mut row = vec![];
        for ch in line.chars() {
            row.push(match ch {
                '.' => Open,
                '|' => Trees,
                '#' => Lumberyard,
                _   => bail!("Can't understand character {}", ch),
            });
        }
        grid.push(row);
    }
    Ok(LumberCollection {
        grid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lumber() {
        let input_str = ".#.#...|#.
.....#|##|
.|..|...#.
..|#.....#
#.#|||#|#|
...#.||...
.|....|...
||...#|.#|
|.||||..|.
...#.|..|.";
        let lumber = parse_lumber(&mut input_str.as_bytes())
            .expect("Couldn't parse lumber collection area");
        assert_eq!(format!("{}", lumber).trim(), input_str);
    }

    #[test]
    fn test_lumber_advance() {
        let input_str = ".#.#...|#.
.....#|##|
.|..|...#.
..|#.....#
#.#|||#|#|
...#.||...
.|....|...
||...#|.#|
|.||||..|.
...#.|..|.";
        let mut lumber = parse_lumber(&mut input_str.as_bytes())
            .expect("Couldn't parse lumber collection area");
        lumber.advance();
        assert_eq!(format!("{}", lumber).trim(), ".......##.
......|###
.|..|...#.
..|#||...#
..##||.|#|
...#||||..
||...|||..
|||||.||.|
||||||||||
....||..|.");
    }

    #[test]
    fn test_lumber_resource_value() {
        let input_str = ".#.#...|#.
.....#|##|
.|..|...#.
..|#.....#
#.#|||#|#|
...#.||...
.|....|...
||...#|.#|
|.||||..|.
...#.|..|.";
        let mut lumber = parse_lumber(&mut input_str.as_bytes())
            .expect("Couldn't parse lumber collection area");
        lumber.advance_multiple(10);
        assert_eq!(lumber.resource_value(), 1147);
    }
}
