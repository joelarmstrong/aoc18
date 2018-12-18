use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::io::BufRead;
use regex::Regex;
use failure::{Error, format_err};

pub fn aoc12(part2: bool) -> Result<(), Error> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let initial_state = parse_initial_state(&line)?;
    stdin.lock().read_line(&mut line)?;
    let rules = parse_rules(&mut stdin.lock())?;
    let mut pc = PlantCells::new(initial_state, rules);
    let num_steps = if part2 { 50000000000 } else { 20 };
    pc.advance_n_steps(num_steps);
    println!("Sum after {} steps: {}", num_steps, pc.sum());
    Ok(())
}

type PlantContext = (bool, bool, bool, bool, bool);

struct PlantCells {
    cells: Vec<bool>,
    start_index: i64,
    rules: HashMap<PlantContext, bool>,
}

impl PlantCells {
    fn new(initial_state: Vec<bool>, rules: HashMap<PlantContext, bool>) -> Self {
        PlantCells {
            cells: initial_state,
            start_index: 0,
            rules,
        }
    }

    fn advance(&mut self) {
        let mut old_state = Vec::new();
        old_state.push(false);
        old_state.push(false);
        old_state.push(false);
        old_state.push(false);
        self.cells.push(false);
        self.cells.push(false);
        self.cells.push(false);
        self.cells.push(false);
        old_state.append(&mut self.cells);
        self.start_index -= 2;
        for context_slice in old_state.as_slice().windows(5) {
            let context = (context_slice[0], context_slice[1], context_slice[2], context_slice[3], context_slice[4]);
            let new_cell = self.rules.get(&context).unwrap_or(&false);
            if *new_cell || !self.cells.is_empty() {
                self.cells.push(*new_cell);
            } else {
                self.start_index += 1;
            }
        }
        // Count and remove any extra trailing elements at the end.
        let extra_elems = self.cells.iter().rev().take_while(|&&c| c == false).count();
        self.cells.resize(self.cells.len() - extra_elems, false);
    }

    fn advance_n_steps(&mut self, n: u64) {
        // Timestep at which we last saw a given configuration.
        let mut last_seen = HashMap::new();
        for i in 0..n {
            self.advance();
            if let Some((j, prev_start)) = last_seen.get(&self.cells) {
                // Found a cycle.
                let cycle_length = i - j;
                let start_delta = self.start_index - prev_start;
                let remainder = (n - i - 1) % cycle_length;
                for _ in 0..remainder {
                    self.advance();
                }
                self.start_index += start_delta * (((n - i - 1) / cycle_length) as i64);
                break;
            }
            last_seen.insert(self.cells.clone(), (i, self.start_index));
        }
    }

    fn sum(&self) -> i64 {
        self.cells.iter().enumerate().map(|(i, c)| if *c { (i as i64) + self.start_index } else { 0 }).sum()
    }
}

impl Display for PlantCells {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for cell in self.cells.iter() {
            write!(f, "{}", if *cell { '#' } else { '.' })?;
        }
        Ok(())
    }
}

fn parse_initial_state(state_line: &str) -> Result<Vec<bool>, Error> {
    let regex = Regex::new(r"initial state: ([#.]+)")?;
    let caps = regex.captures(state_line).ok_or_else(|| format_err!("Can't understand initial state line {}", state_line))?;
    let state_str = caps.get(1).ok_or_else(|| format_err!("No state within state line"))?.as_str();
    Ok(state_str.chars().map(|c| c == '#').collect())
}

fn parse_rules(input: &mut BufRead) -> Result<HashMap<PlantContext, bool>, Error> {
    let regex = Regex::new(r"([.#]{5}) => ([#.])")?;
    let mut rules = HashMap::new();
    for line_res in input.lines() {
        let line = line_res?;
        let caps = regex.captures(&line).ok_or_else(|| format_err!("Can't understand rule line {}", line))?;
        let context_str = caps.get(1).ok_or_else(|| format_err!("Couldn't find context in line {}", line))?.as_str();
        let result_str = caps.get(2).ok_or_else(|| format_err!("Couldn't find result in line {}", line))?.as_str();
        let context_vec: Vec<_> = context_str.chars().map(|c| c == '#').collect();
        let context = (context_vec[0], context_vec[1], context_vec[2], context_vec[3], context_vec[4]);
        let result = result_str == "#";
        rules.insert(context, result);
    }
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    /// This function allows us to assert that a Result is
    /// Ok(expected) without requiring PartialEq on the Error type.
    fn assert_result_ok<T: Debug + PartialEq>(r: Result<T, Error>, expected: T) {
        match r {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("got Err: {}, local backtrace: {}", e, e.backtrace()),
        }
    }

    const STATE: &str = "initial state: #..#.#..##......###...###";

    const RULES: &str = "...## => #
..#.. => #
.#... => #
.#.#. => #
.#.## => #
.##.. => #
.#### => #
#.#.# => #
#.### => #
##.#. => #
##.## => #
###.. => #
###.# => #
####. => #
..... => .";

    #[test]
    fn test_parse_initial_state() {
        assert_result_ok(parse_initial_state(STATE), vec![
            true,  false, false, true,  false,
            true,  false, false, true,  true,
            false, false, false, false, false,
            false, true,  true,  true,  false,
            false, false, true,  true,  true,
        ]);
    }

    #[test]
    fn test_parse_rules() {
        let rules_map: HashMap<PlantContext, bool> = vec![
            ((false, false, false, true,  true),  true),
            ((false, false, true,  false, false), true),
            ((false, true,  false, false, false), true),
            ((false, true,  false, true,  false), true),
            ((false, true,  false, true,  true),  true),
            ((false, true,  true,  false, false), true),
            ((false, true,  true,  true,  true),  true),
            ((true,  false, true,  false, true),  true),
            ((true,  false, true,  true,  true),  true),
            ((true,  true,  false, true,  false), true),
            ((true,  true,  false, true,  true),  true),
            ((true,  true,  true,  false, false), true),
            ((true,  true,  true,  false, true),  true),
            ((true,  true,  true,  true,  false), true),
            ((false, false, false, false, false), false),
        ].into_iter().collect();
        assert_result_ok(parse_rules(&mut RULES.as_bytes()), rules_map);
    }

    #[test]
    fn test_plantcells_display() {
        let state = parse_initial_state(STATE).expect("Couldn't parse state");
        let rules = parse_rules(&mut RULES.as_bytes()).expect("Couldn't parse rules");
        let pc = PlantCells::new(state, rules);
        assert_eq!(format!("{}", pc), "#..#.#..##......###...###");
    }

    #[test]
    fn test_plantcells_advance() {
        let state = parse_initial_state(STATE).expect("Couldn't parse state");
        let rules = parse_rules(&mut RULES.as_bytes()).expect("Couldn't parse rules");
        let mut pc = PlantCells::new(state, rules);
        pc.advance();
        assert_eq!(format!("{}", pc), "#...#....#.....#..#..#..#");
        assert_eq!(pc.start_index, 0);
        pc.advance_n_steps(19);
        assert_eq!(format!("{}", pc), "#....##....#####...#######....#.#..##");
        assert_eq!(pc.start_index, -2);
    }

    #[test]
    fn test_plantcells_advance_giant_steps() {
        let state = parse_initial_state(STATE).expect("Couldn't parse state");
        let rules = parse_rules(&mut RULES.as_bytes()).expect("Couldn't parse rules");
        let mut pc1 = PlantCells::new(state.clone(), rules.clone());
        let mut pc2 = PlantCells::new(state, rules);
        pc1.advance_n_steps(867);
        for _ in 0..867 {
            pc2.advance();
        }
        assert_eq!(format!("{}", pc1), format!("{}", pc2));
        assert_eq!(pc1.start_index, pc2.start_index);
        assert_eq!(pc1.sum(), pc2.sum());
    }

    #[test]
    fn test_plantcells_sum() {
        let state = parse_initial_state(STATE).expect("Couldn't parse state");
        let rules = parse_rules(&mut RULES.as_bytes()).expect("Couldn't parse rules");
        let mut pc = PlantCells::new(state, rules);
        pc.advance_n_steps(20);
        assert_eq!(pc.sum(), 325);
    }
}
