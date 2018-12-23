use std::collections::HashSet;
use std::cmp::min;
use std::io;
use std::io::BufRead;
use std::ops::RangeInclusive;
use failure::{Error, ensure, format_err};
use regex::Regex;

pub fn aoc17(part2: bool) -> Result<(), Error> {
    let mut reservoir = parse_veins(&mut io::stdin().lock())?;
    if part2 {
    } else {
        reservoir.fill_with_water();
        println!("Watered squares: {}", reservoir.count_water());
    }
    Ok(())
}

#[derive(PartialEq, Debug, Clone)]
enum ReservoirContents {
    Clay,
    Sand,
    Spring,
    Water,
    /// Sand that water has already passed through.
    DampSand,
}

use self::ReservoirContents::*;

#[derive(PartialEq, Debug, Clone)]
enum WaterBoundaries {
    // Bounded on the left and right by clay walls, so the water
    // occupies the given range.
    Bounded(RangeInclusive<usize>),
    // Spilling over on one side, and possibly both, at the given
    // locations.
    Spill(usize, Option<usize>),
}

use self::WaterBoundaries::*;

struct Reservoir {
    y_min: usize,
    grid: Vec<Vec<ReservoirContents>>,
}

impl Reservoir {
    fn new() -> Self {
        let mut res = Reservoir {
            grid: vec![],
            y_min: 0,
        };
        res.extend_to(500, 0);
        *res.get_mut(500, 0).unwrap() = Spring;
        res
    }

    fn get(&self, x: usize, y: usize) -> Option<&ReservoirContents> {
        self.grid.get(y).and_then(|r| r.get(x))
    }

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut ReservoirContents> {
        self.grid.get_mut(y).and_then(|r| r.get_mut(x))
    }

    fn extend_to(&mut self, x: usize, y: usize) {
        while self.grid.len() <= y {
            self.grid.push(vec![]);
        }
        let row = self.grid.get_mut(y).unwrap();
        if row.len() <= x {
            row.resize(x + 1, Sand);
        }
    }

    fn fill_with_water(&mut self) {
        let mut source_stack = vec![(500, 0)];
        let mut done_sources = HashSet::new();
        while source_stack.len() != 0 {
            let (source_x, source_y) = source_stack.pop().unwrap();
            println!("source: {} {}", source_x, source_y);
            if self.get(source_x, source_y) == Some(&Water) {
                // Overflowed back up to the source.
                done_sources.insert((source_x, source_y));
                continue;
            }
            let mut cur_y = source_y;
            loop {
                println!("{} {}", source_x, cur_y);
                if cur_y >= self.grid.len() {
                    // Out of bounds.
                    done_sources.insert((source_x, source_y));
                    break;
                }
                self.extend_to(source_x, cur_y);
                *self.get_mut(source_x, cur_y).unwrap() = DampSand;
                let contents_below = self.get(source_x, cur_y + 1);
                if contents_below == Some(&Clay) || contents_below == Some(&Water) {
                    let boundary = self.get_boundary(source_x, cur_y);
                    match boundary {
                        Bounded(range) => {
                            println!("bounded");
                            for x in range {
                                *self.get_mut(x, cur_y).unwrap() = Water;
                            }
                            if !done_sources.contains(&(source_x, source_y)) {
                                source_stack.push((source_x, source_y));
                            } else {
                                println!("Attempt to revisit source {}, {}", source_x, source_y);
                            }
                            break;
                        },
                        Spill(spill1, spill2_opt) => {
                            println!("spill {} {:?}", spill1, spill2_opt);
                            if !done_sources.contains(&(spill1, cur_y)) {
                                source_stack.push((spill1, cur_y));
                            } else {
                                println!("Attempt to revisit source {}, {}", spill1, cur_y);
                            }
                            if let Some(spill2) = spill2_opt {
                                if !done_sources.contains(&(spill2, cur_y)) {
                                    source_stack.push((spill2, cur_y));
                                } else {
                                    println!("Attempt to revisit source {}, {}", spill2, cur_y);
                                }
                            }
                            break;
                        },
                    }
                }
                cur_y += 1;
            }
        }
    }

    fn get_boundary(&mut self, x: usize, y: usize) -> WaterBoundaries {
        let mut cur_x = x;
        let mut spills = vec![];
        let mut wall_start = None;
        // First check to the left.
        loop {
            assert!(x != 0, "Ran into left edge when seeking for boundaries");
            if Some(&Clay) == self.get(cur_x, y) {
                wall_start = Some(cur_x);
                break;
            } else {
                self.extend_to(cur_x, y);
                *self.get_mut(cur_x, y).unwrap() = DampSand;
            }
            let floor = self.get(cur_x, y + 1);
            if floor != Some(&Clay) && floor != Some(&Water) {
                spills.push(cur_x);
                break;
            }
            cur_x -= 1;
        }
        cur_x = x;
        // Now check to the right.
        let mut wall_end = None;
        loop {
            if Some(&Clay) == self.get(cur_x, y) {
                wall_end = Some(cur_x);
                break;
            } else {
                self.extend_to(cur_x, y);
                *self.get_mut(cur_x, y).unwrap() = DampSand;
            }
            let floor = self.get(cur_x, y + 1);
            if floor != Some(&Clay) && floor != Some(&Water) {
                spills.push(cur_x);
                break;
            }
            cur_x += 1;
        }
        println!("wall_start {:?} wall_end {:?}", wall_start, wall_end);
        if let (Some(start), Some(end)) = (wall_start, wall_end) {
            return Bounded(start + 1..=end - 1);
        }
        if spills.len() == 1 {
            return Spill(spills[0], None);
        }
        if spills.len() == 2 {
            return Spill(spills[0], Some(spills[1]));
        }
        unreachable!();
    }

    fn count_water(&self) -> usize {
        let mut count = 0;
        for row in self.grid.iter().skip(self.y_min) {
            for column in row {
                if column == &Water || column == &DampSand {
                    count += 1;
                }
            }
        }
        count
    }
}

fn parse_range(range_str: &str) -> Result<RangeInclusive<usize>, Error>{
    let split = range_str.split("..").map(|s| s.parse::<usize>()).collect::<Result<Vec<_>, _>>()?;
    if split.len() > 1 {
        ensure!(split.len() == 2, "More fields than expected in range");
        Ok(split[0]..=split[1])
    } else {
        Ok(split[0]..=split[0])
    }
}

fn parse_veins(input: &mut impl BufRead) -> Result<Reservoir, Error> {
    let mut reservoir = Reservoir::new();
    let x_regex=Regex::new(r"x=[0-9.]+")?;
    let y_regex=Regex::new(r"y=[0-9.]+")?;
    let mut y_min = std::usize::MAX;
    for line_res in input.lines() {
        let line = line_res?;
        let x_match = x_regex.find(&line).ok_or_else(|| format_err!("Didn't find x in line {}", line))?.as_str();
        let x_range = parse_range(x_match.trim_start_matches("x="))?;
        let y_match = y_regex.find(&line).ok_or_else(|| format_err!("Didn't find y in line {}", line))?.as_str();
        let y_range = parse_range(y_match.trim_start_matches("y="))?;
        let max_x = *x_range.end();
        y_min = min(y_min, *y_range.start());
        for y in y_range {
            reservoir.extend_to(max_x, y);
            for x in x_range.clone() {
                *reservoir.get_mut(x, y).unwrap() = Clay;
            }
        }
    }
    reservoir.y_min = y_min;
    Ok(reservoir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_veins() {
        let input = "x=495, y=2..7
y=7, x=495..501
x=501, y=3..7
x=498, y=2..4
x=506, y=1..2
x=498, y=10..13
x=504, y=10..13
y=13, x=498..504";
        let reservoir = parse_veins(&mut input.as_bytes()).expect("Couldn't parse");
        assert_eq!(reservoir.get(495, 3), Some(&Clay));
        assert_eq!(reservoir.get(500, 13), Some(&Clay));
        assert_eq!(reservoir.get(500, 12), Some(&Sand));
        assert_eq!(reservoir.get(500, 0), Some(&Spring));
    }

    #[test]
    fn test_fill_with_water() {
        let input = "x=495, y=2..7
y=7, x=495..501
x=501, y=3..7
x=498, y=2..4
x=506, y=1..2
x=498, y=10..13
x=504, y=10..13
y=13, x=498..504";
        let mut reservoir = parse_veins(&mut input.as_bytes()).expect("Couldn't parse");
        reservoir.fill_with_water();
        assert_eq!(reservoir.count_water(), 57);
    }
}
