use crate::aoc6::Coord;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use std::fmt;
use std::io;
use std::io::BufRead;
use failure::{Error, bail, format_err};

pub fn aoc22(part2: bool) -> Result<(), Error> {
    let mut cave = parse_cave(&mut io::stdin().lock())?;
    if part2 {
        println!("Time taken to : {}", cave.time_to_find_target()?);
    } else {
        println!("Risk level: {}", cave.risk_level());
    }
    Ok(())
}

#[derive(PartialEq, Debug)]
struct Cave {
    depth: usize,
    cells: Vec<Vec<ErosionLevel>>,
    scores: Vec<Vec<usize>>,
    target: Coord,
    fill_x: usize,
    fill_y: usize,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum ErosionLevel {
    Wet,
    Rocky,
    Narrow,
    Mouth,
    Target,
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Clone, Copy, Hash)]
enum Tool {
    Torch,
    ClimbingGear,
    Neither,
}

const TOOLS: [Tool; 3] = [Tool::Torch, Tool::ClimbingGear, Tool::Neither];

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Clone)]
struct PathFrontier {
    heuristic_cost: u64,
    best_cost: u64,
    tool: Tool,
    x: usize,
    y: usize,
    tool_switches: u32,
}

impl Cave {
    fn new(depth: usize, target: Coord) -> Self {
        Cave {
            depth,
            target,
            cells: Vec::new(),
            scores: Vec::new(),
            fill_x: 0,
            fill_y: 0,
        }
    }

    fn fill(&mut self, x: usize, y: usize) {
        let x = x.max(self.fill_x);
        let y = y.max(self.fill_y);
        if x <= self.fill_x && y <= self.fill_y {
            return;
        }
        if self.cells.len() < y + 1 {
            self.cells.resize(y + 1, vec![]);
            self.scores.resize(y + 1, vec![]);
        }
        for y in 0..=y {
            if self.cells[y].len() < x + 1 {
                self.cells[y].resize(x + 1, ErosionLevel::Wet);
            }
            self.scores[y].resize(x + 1, 0);
            for x in 0..=x {
                if self.scores[y][x] != 0 {
                    continue;
                }
                self.scores[y][x] = 
                    (match (x, y) {
                        (0, 0) => 0,
                        (0, _) => y * 48271,
                        (_, 0) => x * 16807,
                        (x, y) if x == self.target.x as usize &&
                                  y == self.target.y as usize => 0,
                        _ => self.scores[y][x - 1] * self.scores[y - 1][x],
                    } + self.depth) % 20183;
                self.cells[y][x] = match self.scores[y][x] % 3 {
                    0 => ErosionLevel::Rocky,
                    1 => ErosionLevel::Wet,
                    2 => ErosionLevel::Narrow,
                    _ => unreachable!(),
                }
            }
        }
        self.cells[0][0] = ErosionLevel::Mouth;
        self.cells[self.target.y as usize][self.target.x as usize] =
            ErosionLevel::Target;
        self.fill_x = self.fill_x.max(x);
        self.fill_y = self.fill_y.max(y);
    }

    fn risk_level(&mut self) -> usize {
        self.fill(self.target.x as usize, self.target.y as usize);
        let mut risk = 0;
        for y in 0..=(self.target.y as usize) {
            for x in 0..=(self.target.x as usize) {
                risk += match self.cells[y][x] {
                    ErosionLevel::Rocky => 0,
                    ErosionLevel::Wet => 1,
                    ErosionLevel::Narrow => 2,
                    _ => 0,
                }
            }
        }
        risk
    }

    fn get(&mut self, x: usize, y: usize) -> ErosionLevel {
        self.fill(x, y);
        self.cells[y][x]
    }

    fn time_to_find_target(&mut self) -> Result<u64, Error> {
        self.fill(self.target.x as usize, self.target.y as usize);
        // We use A* on an augmented graph with 3 nodes per room in
        // the cave (representing the 3 tools). There exists an edge
        // with cost 1 between each adjacent room with the same
        // tool. Between each set of tools in the same room there is
        // an edge with cost 7. We want to find the shortest path to
        // the node representing the target with ClimbingGear.
        let target_x = self.target.x;
        let target_y = self.target.y;
        let distance_to_target = |x,y| {
            (((x as i64) - target_x).abs() +
            ((y as i64) - target_y).abs()) as u64
        };
        let mut best_score: HashMap<(usize, usize, Tool), u64> = HashMap::new();
        best_score.insert((0, 0, Tool::Torch), 0);
        let mut priority_queue: BinaryHeap<Reverse<PathFrontier>> =
            BinaryHeap::new();
        priority_queue.push(Reverse(PathFrontier {
            best_cost: 0,
            heuristic_cost: distance_to_target(0, 0),
            tool: Tool::Torch,
            x: 0,
            y: 0,
            tool_switches: 0,
        }));
        while let Some(Reverse(frontier)) = priority_queue.pop() {
            if frontier.x == self.target.x as usize &&
               frontier.y == self.target.y as usize &&
               frontier.tool == Tool::Torch {
               return Ok(frontier.best_cost);
            }
            let mut neighbors: [Option<(usize, usize)>; 4] = [
                Some((frontier.x + 1, frontier.y)),
                Some((frontier.x, frontier.y + 1)),
                None,
                None,
            ];
            if frontier.x != 0 {
                neighbors[2] = Some((frontier.x - 1, frontier.y));
            }
            if frontier.y != 0 {
                neighbors[3] = Some((frontier.x, frontier.y - 1));
            }
            for &(neighbor_x, neighbor_y) in neighbors.iter().flatten() {
                // Remove edges that shouldn't exist
                match self.get(neighbor_x, neighbor_y) {
                    ErosionLevel::Rocky | ErosionLevel::Target if frontier.tool == Tool::Neither => continue,
                    ErosionLevel::Wet if frontier.tool == Tool::Torch => continue,
                    ErosionLevel::Narrow if frontier.tool == Tool::ClimbingGear => continue,
                    _ => {},
                }
                let best = best_score.get(&(neighbor_x, neighbor_y, frontier.tool)).cloned().unwrap_or(frontier.best_cost + 10);
                if best <= frontier.best_cost + 1 {
                    continue;
                }
                best_score.insert((neighbor_x, neighbor_y, frontier.tool), frontier.best_cost + 1);
                priority_queue.push(Reverse(PathFrontier {
                    best_cost: frontier.best_cost + 1,
                    heuristic_cost: frontier.best_cost + 1 + distance_to_target(neighbor_x, neighbor_y),
                    tool: frontier.tool,
                    x: neighbor_x,
                    y: neighbor_y,
                    tool_switches: frontier.tool_switches,
                }));
            }
            for &tool in TOOLS.iter() {
                if tool == frontier.tool {
                    continue;
                }
                match self.get(frontier.x, frontier.y) {
                    ErosionLevel::Rocky if tool == Tool::Neither => continue,
                    ErosionLevel::Wet if tool == Tool::Torch => continue,
                    ErosionLevel::Narrow if tool == Tool::ClimbingGear => continue,
                    ErosionLevel::Target if tool != Tool::Torch => continue,
                    _ => {},
                }
                let best = best_score.get(&(frontier.x, frontier.y, tool)).cloned().unwrap_or(frontier.best_cost + 10);
                if best <= frontier.best_cost + 7 {
                    continue;
                }
                best_score.insert((frontier.x, frontier.y, tool), frontier.best_cost + 7);
                // Add edge to change tools
                priority_queue.push(Reverse(PathFrontier {
                    best_cost: frontier.best_cost + 7,
                    heuristic_cost: frontier.best_cost + 7 + distance_to_target(frontier.x, frontier.y),
                    tool: tool,
                    x: frontier.x,
                    y: frontier.y,
                    tool_switches: frontier.tool_switches + 1,
                }));
            }
        }
        Err(format_err!("Can't find target"))
    }
}

impl fmt::Display for Cave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.cells.len() {
            for x in 0..self.cells[y].len() {
                let ch = match self.cells[y][x] {
                    ErosionLevel::Rocky => '.',
                    ErosionLevel::Wet => '=',
                    ErosionLevel::Narrow => '|',
                    ErosionLevel::Target => 'T',
                    ErosionLevel::Mouth => 'M',
                };
                write!(f, "{}", ch)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn parse_cave(input: &mut impl BufRead) -> Result<Cave, Error> {
    let mut depth = None;
    let mut target = None;
    for line_res in input.lines() {
        let line = line_res?;
        let fields: Vec<_> = line.split(" ").collect();
        match fields[0] {
            "depth:" => depth = Some(fields[1].parse()?),
            "target:" => {
                let coords: Vec<_> = fields[1].split(",").collect();
                target = Some(Coord { x: coords[0].parse()?,
                                      y: coords[1].parse()? });
            },
            e => bail!("unknown type {}", e),
        }
    }
    Ok(Cave::new(depth.ok_or_else(|| format_err!("no depth found"))?,
                 target.ok_or_else(|| format_err!("no target found"))?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cave() -> Result<(), Error> {
        let input_str = "depth: 5355
target: 14,796
";
        assert_eq!(parse_cave(&mut input_str.as_bytes())?, Cave::new(
            5355, Coord { x: 14, y: 796 }));
        Ok(())
    }

    #[test]
    fn test_cave_fill() {
        let mut cave = Cave::new(510, Coord { x: 10, y: 10 });
        cave.fill(15, 15);
        assert_eq!(format!("{}", cave), "M=.|=.|.|=.|=|=.
.|=|=|||..|.=...
.==|....||=..|==
=.|....|.==.|==.
=|..==...=.|==..
=||.=.=||=|=..|=
|.=.===|||..=..|
|..==||=.|==|===
.=..===..=|.|||.
.======|||=|=.|=
.===|=|===T===||
=|||...|==..|=.|
=.=|=.=..=.||==|
||=|=...|==.=|==
|=.=||===.|||===
||.|==.|.|.||=||
");
    }

    #[test]
    fn test_risk_level() {
        let mut cave = Cave::new(510, Coord { x: 10, y: 10 });
        assert_eq!(cave.risk_level(), 114);
    }

    #[test]
    fn test_time_to_find_target() -> Result<(), Error> {
        let mut cave = Cave::new(510, Coord { x: 10, y: 10 });
        assert_eq!(cave.time_to_find_target()?, 45);
        Ok(())
    }
}
