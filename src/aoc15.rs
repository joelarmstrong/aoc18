use std::mem;
use std::collections::{HashSet, HashMap, VecDeque};
use std::fmt;
use std::fmt::Display;
use std::io;
use std::io::BufRead;
use failure::{Error, bail};

pub fn aoc15(part2: bool) -> Result<(), Error> {
    let mut cavern = parse_cavern(&mut io::stdin().lock())?;
    if part2 {
    } else {
        cavern.advance_till_finish();
        println!("Combat outcome: {}", cavern.outcome());
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum CavernContents {
    Wall,
    Open,
    Unreachable,
    Occupied(Unit),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum UnitType {
    Elf,
    Goblin,
}

use self::CavernContents::*;
use self::UnitType::*;

#[derive(Debug, PartialEq, Eq, Hash)]
struct Unit {
    team: UnitType,
    hp: u8,
    attack_power: u8,
    id: u64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Coord {
    x: usize,
    y: usize,
}

struct Attack {
    pos: Coord,
    damage: u8,
}

struct Turn {
    movement: Option<Coord>,
    attack: Option<Attack>,
}

impl Unit {
    fn new(team: UnitType, id: u64) -> Self {
        Unit {
            team,
            attack_power: 3,
            hp: 200,
            id: id
        }
    }

    fn take_turn(&self, layout: &Vec<Vec<CavernContents>>, x: usize, y: usize) -> Turn {
        let mut attack = self.attack(layout, x, y);
        let mut movement = None;
        if attack.is_none() {
            movement = self.movement(layout, x, y);
            if let Some(ref coord) = movement {
                // Try attacking from our new position now that we've moved.
                attack = self.attack(layout, coord.x, coord.y);
            }
        }
        Turn { movement: movement,
               attack: attack, }
    }

    fn movement(&self, layout: &Vec<Vec<CavernContents>>, x: usize, y: usize) -> Option<Coord> {
        let paths = bfs(layout, x, y);
        // Find potential targets and choose the closest
        let targets = self.desired_squares(layout);
        targets.iter().filter_map(|c| shortest_path(&paths, *c)).min_by_key(|p| p.len()).map(|p| p[0])
    }

    fn desired_squares(&self, layout: &Vec<Vec<CavernContents>>) -> Vec<Coord> {
        let mut good_moves = vec![];
        for (y, row) in layout.iter().enumerate() {
            for (x, _column) in row.iter().enumerate() {
                if let Some(_attack) = self.attack(layout, x, y) {
                    good_moves.push(Coord { x, y });
                }
            }
        }
        good_moves
    }

    fn attack(&self, layout: &Vec<Vec<CavernContents>>, x: usize, y: usize) -> Option<Attack> {
        let other_team = if self.team == Elf { Goblin } else { Elf };
        let coords = get_adjacencies(layout, x, y);
        let mut potential_targets = vec![];
        for coord in coords.into_iter() {
            match layout.get(coord.y).and_then(|r| r.get(coord.x)) {
                Some(Occupied(u)) => {
                    if u.team == other_team {
                        potential_targets.push((u, coord));
                    }
                }
                _ => {},
            }
        }
        potential_targets.into_iter()
            .min_by_key(|(u, _coord)| u.hp)
            .map(|(_, coord)| Attack { pos: coord, damage: self.attack_power })
    }
}

/// Find shortest paths from (x, y) in the cavern. Returns a hash
/// representing the child -> parent relationships in the BFS tree.
fn bfs(layout: &Vec<Vec<CavernContents>>, x: usize, y: usize) -> HashMap<Coord, Coord> {
    let mut parent = HashMap::new();
    let mut discovered: HashSet<Coord> = HashSet::new();
    let mut queue: VecDeque<Coord> = VecDeque::new();
    queue.push_back(Coord { x, y });
    discovered.insert(Coord { x, y });
    while !queue.is_empty() {
        let node = queue.pop_front().unwrap();
        for adjacency in get_valid_adjacencies(layout, &node) {
            if !discovered.contains(&adjacency) {
                parent.insert(adjacency, node);
                discovered.insert(adjacency);
                queue.push_back(adjacency);
            }
        }
    }
    parent
}

fn shortest_path(parent: &HashMap<Coord, Coord>, destination: Coord) -> Option<Vec<Coord>> {
    let mut path = vec![];
    let mut cur_point = destination;
    while let Some(coord) = parent.get(&cur_point) {
        path.push(cur_point);
        cur_point = *coord;
    }
    path.reverse();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Get all valid (unoccupied) adjacent coordinates in reading order.
fn get_valid_adjacencies(layout: &Vec<Vec<CavernContents>>, coord: &Coord) -> Vec<Coord> {
    get_adjacencies(layout, coord.x, coord.y).into_iter().filter(|c| layout[c.y][c.x] == Open).collect()
}

/// Get all adjacencies that don't go off the cavern data, in reading order.
fn get_adjacencies(layout: &Vec<Vec<CavernContents>>, x: usize, y: usize) -> Vec<Coord> {
    let x_1 = x.checked_sub(1);
    let x_2 = if x < layout[y].len() - 1 { Some(x + 1) } else { None };
    let y_1 = y.checked_sub(1);
    let y_2 = if y < layout.len() { Some(y + 1) } else { None };

    let mut adjacencies = vec![];
    if let Some(y_) = y_1 {
        adjacencies.push(Coord { x, y: y_ });
    }
    if let Some(x_) = x_1 {
        adjacencies.push(Coord { x: x_, y });
    }
    if let Some(x_) = x_2 {
        adjacencies.push(Coord { x: x_, y });
    }
    if let Some(y_) = y_2 {
        adjacencies.push(Coord { x, y: y_ });
    }
    adjacencies
}

struct Cavern {
    layout: Vec<Vec<CavernContents>>,
    turns: u64,
}

impl Cavern {
    /// Simulate one full round.
    fn advance(&mut self) -> bool {
        println!("beginning {}: {}", self.turns, self);
        // Keep track of IDs of units which have taken their turn, so
        // we don't advance them twice if they move down or right.
        let mut units_done: HashSet<u64> = HashSet::new();
        for y in 0..self.layout.len() {
            for x in 0..self.layout[y].len() {
                let mut turn_opt = None;
                if let Occupied(ref unit) = self.layout[y][x] {
                    if !units_done.contains(&unit.id) {
                        turn_opt = Some(unit.take_turn(&self.layout, x, y));
                        units_done.insert(unit.id);
                    }
                }
                if self.is_done() {
                    println!("quitting round early {}", self);
                    return true;
                }
                if let Some(turn) = turn_opt {
                    self.apply_turn(x, y, turn);
                }
            }
        }
        self.turns += 1;
        println!("{}: {}", self.turns, self);
        self.is_done()
    }

    fn is_done(&self) -> bool {
        let mut teams = HashSet::new();
        for row in &self.layout {
            for column in row {
                match column {
                    Occupied(unit) => teams.insert(&unit.team),
                    _ => false,
                };
            }
        }
        teams.len() == 1
    }

    fn advance_till_finish(&mut self) {
        while !self.advance() {
        }
    }

    fn apply_turn(&mut self, x: usize, y: usize, turn: Turn) {
        if let Some(coord) = turn.movement {
            let unit = mem::replace(&mut self.layout[y][x], Open);
            assert!(self.layout[coord.y][coord.x] == Open);
            self.layout[coord.y][coord.x] = unit;
        }
        if let Some(attack) = turn.attack {
            let mut delete = false;
            if let Occupied(ref mut unit) = self.layout[attack.pos.y][attack.pos.x] {
                unit.hp = unit.hp.saturating_sub(attack.damage);
                if unit.hp == 0 {
                    delete = true;
                }
            } else {
                assert!(false, "Attacking a non-unit");
            }
            if delete {
                self.layout[attack.pos.y][attack.pos.x] = Open;
            }
        }
    }

    fn outcome(&self) -> u64 {
        println!("{} {}", self.turns, self.layout.iter().map(|r| r.iter().map(|e| match e { Occupied(u) => u.hp as u64, _ => 0 }).sum::<u64>()).sum::<u64>());
        self.turns * self.layout.iter().map(|r| r.iter().map(|e| match e { Occupied(u) => u.hp as u64, _ => 0 }).sum::<u64>()).sum::<u64>()
    }
}

fn parse_cavern(input: &mut BufRead) -> Result<Cavern, Error> {
    let mut layout = vec![];
    // Unique ID for tracking units.
    let mut id = 0;
    for line_res in input.lines() {
        let line = line_res?;
        let row_layout = line.chars().map(|c| {
            id += 1;
            Ok(match c {
                'G' => Occupied(Unit::new(Goblin, id)),
                'E' => Occupied(Unit::new(Elf, id)),
                '.' => Open,
                '#' => Wall,
                ' ' => Unreachable,
                _   => bail!("Don't understand square type {}", c),
            })}).collect::<Result<Vec<_>, _>>()?;
        layout.push(row_layout);
    }
    Ok(Cavern {
        layout,
        turns: 0,
    })
}

impl Display for CavernContents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Wall => '#',
            Open => '.',
            Unreachable => ' ',
            Occupied(u) => if u.team == Elf { 'E' } else { 'G' },
        };
        write!(f, "{}", c)
    }
}

impl Display for Cavern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.layout {
            for item in row {
                write!(f, "{}", item)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cavern() {
        let cavern_str = "
#########
#G..G..G#
#.......#
#.......#
#G..E..G#
#.......#
#.......#
#G..G..G#
#########
";
        let cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        assert_eq!(format!("{}", cavern), cavern_str);
    }

    #[test]
    fn test_shortest_path() {
        let cavern_str = "#######
#.E...#
#.....#
#...G.#
#######
";
        let cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        let paths = bfs(&cavern.layout, 2, 1);
        let path = shortest_path(&paths, Coord { x: 4, y: 2});
        assert_eq!(path, Some(vec![
            Coord { x: 3, y: 1 },
            Coord { x: 4, y: 1 },
            Coord { x: 4, y: 2 },
        ]));
    }

    #[test]
    fn test_movement() {
        let cavern_str = "#########
#G..G..G#
#.......#
#.......#
#G..E..G#
#.......#
#.......#
#G..G..G#
#########
";
        let cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        if let Occupied(ref gob) = cavern.layout[1][1] {
            // Check goblin at 1,1
            let movement = gob.movement(&cavern.layout, 1, 1);
            assert_eq!(movement, Some(Coord { x: 2, y: 1 }));
            // Check goblin at 1, 4 (we can still use the first goblin
            // since it doesn't know its x/y coords)
            let movement = gob.movement(&cavern.layout, 1, 4);
            assert_eq!(movement, Some(Coord { x: 2, y: 4 }));
        } else {
            assert!(false, "unoccupied space");
        }
        if let Occupied(ref elf) = cavern.layout[4][4] {
            let movement = elf.movement(&cavern.layout, 4, 4);
            assert_eq!(movement, Some(Coord { x: 4, y: 3 }));
        } else {
            assert!(false, "unoccupied space");
        }
    }

    #[test]
    fn test_advance() {
        let cavern_str = "#######
#.G...#
#...EG#
#.#.#G#
#..G#E#
#.....#
#######";
        let mut cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        cavern.advance();
        assert_eq!(format!("{}", cavern), "#######
#..G..#
#...EG#
#.#G#G#
#...#E#
#.....#
#######
");
        cavern.advance_till_finish();
        assert_eq!(format!("{}", cavern), "#######
#G....#
#.G...#
#.#.#G#
#...#.#
#....G#
#######
");
    }

    #[test]
    fn test_outcomes() {
        let cavern_str = "#######
#.G...#
#...EG#
#.#.#G#
#..G#E#
#.....#
#######";
        let mut cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        cavern.advance_till_finish();
        //assert_eq!(cavern.outcome(), 27730);

        let cavern_str = "#######
#G..#E#
#E#E.E#
#G.##.#
#...#E#
#...E.#
#######
";
        let mut cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        cavern.advance_till_finish();
        println!("{}", cavern);
        assert_eq!(cavern.outcome(), 36334);

        let cavern_str = "#######
#E..EG#
#.#G.E#
#E.##E#
#G..#.#
#..E#.#
#######";
        let mut cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        cavern.advance_till_finish();
        assert_eq!(cavern.outcome(), 39514);

        let cavern_str = "#######
#E.G#.#
#.#G..#
#G.#.G#
#G..#.#
#...E.#
#######";
        let mut cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        cavern.advance_till_finish();
        assert_eq!(cavern.outcome(), 27755);

        let cavern_str = "#######
#.E...#
#.#..G#
#.###.#
#E#G#G#
#...#G#
#######";
        let mut cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        cavern.advance_till_finish();
        assert_eq!(cavern.outcome(), 28944);

        let cavern_str = "#########
#G......#
#.E.#...#
#..##..G#
#...##..#
#...#...#
#.G...G.#
#.....G.#
#########";
        let mut cavern = parse_cavern(&mut cavern_str.as_bytes()).expect("Couldn't parse cavern");
        cavern.advance_till_finish();
        assert_eq!(cavern.outcome(), 18740);
    }
}
