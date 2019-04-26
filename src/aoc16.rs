use std::collections::{HashSet, HashMap};
use std::io;
use std::io::BufRead;
use failure::{Error, ensure};
use regex::Regex;

pub fn aoc16(part2: bool) -> Result<(), Error> {
    let (samples, instructions) = parse_input(&mut io::stdin().lock())?;
    if part2 {
        let assignments = opcode_assignments(&samples).expect("Can't unambiguously assign opcodes");
        let mut cpu = CPU::new(vec![0, 0, 0, 0]);
        for instruction in instructions {
            let op = assignments[&instruction[0]];
            cpu.apply_op(&op, instruction[1], instruction[2], instruction[3]);
        }
        println!("Value in register 0: {}", cpu.registers[0]);
    } else {
        let count = samples.iter()
            .map(|s| valid_ops(&s.before, &s.instruction, &s.after, ALL_OPS.iter()).len())
            .filter(|&n| n >= 3)
            .count();
        println!("Samples that behave like 3 or more opcodes: {}", count);
    }
    Ok(())
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Opcode {
    AddR,
    AddI,
    MulR,
    MulI,
    BanR,
    BanI,
    BorR,
    BorI,
    SetR,
    SetI,
    GtIR,
    GtRI,
    GtRR,
    EqIR,
    EqRI,
    EqRR,
}

use self::Opcode::*;

pub struct CPU {
    pub registers: Vec<usize>,
}

impl CPU {
    pub fn new(registers: Vec<usize>) -> Self {
        Self {
            registers,
        }
    }

    pub fn apply_op(&mut self, op: &Opcode, arg1: usize, arg2: usize, arg3: usize) {
        self.registers[arg3] = match op {
            AddR => self.registers[arg1] + self.registers[arg2],
            AddI => self.registers[arg1] + arg2,
            MulR => self.registers[arg1] * self.registers[arg2],
            MulI => self.registers[arg1] * arg2,
            BanR => self.registers[arg1] & self.registers[arg2],
            BanI => self.registers[arg1] & arg2,
            BorR => self.registers[arg1] | self.registers[arg2],
            BorI => self.registers[arg1] | arg2,
            SetR => self.registers[arg1],
            SetI => arg1,
            GtIR => if arg1 > self.registers[arg2] { 1 } else { 0 },
            GtRI => if self.registers[arg1] > arg2 { 1 } else { 0 },
            GtRR => if self.registers[arg1] > self.registers[arg2] { 1 } else { 0 },
            EqIR => if arg1 == self.registers[arg2] { 1 } else { 0 },
            EqRI => if self.registers[arg1] == arg2 { 1 } else { 0 },
            EqRR => if self.registers[arg1] == self.registers[arg2] { 1 } else { 0 },
        }
    }
}

const ALL_OPS: [Opcode; 16] = [AddR,
                               AddI,
                               MulR,
                               MulI,
                               BanR,
                               BanI,
                               BorR,
                               BorI,
                               SetR,
                               SetI,
                               GtIR,
                               GtRI,
                               GtRR,
                               EqIR,
                               EqRI,
                               EqRR,];

fn valid_ops<'a>(before: &Vec<usize>, instruction: &Vec<usize>, after: &Vec<usize>, possibilities: impl Iterator<Item = &'a Opcode>) -> Vec<Opcode> {
    let arg1 = instruction[1];
    let arg2 = instruction[2];
    let arg3 = instruction[3];
    let mut valid_ops = vec![];
    for op in possibilities {
        let mut cpu = CPU::new(before.clone());
        cpu.apply_op(op, arg1, arg2, arg3);
        if &cpu.registers == after {
            valid_ops.push(*op);
        }
    }
    valid_ops
}

#[derive(Debug, PartialEq)]
struct Sample {
    before: Vec<usize>,
    instruction: Vec<usize>,
    after: Vec<usize>,
}

fn parse_input(input: &mut BufRead) -> Result<(Vec<Sample>, Vec<Vec<usize>>), Error> {
    let sample_regex = Regex::new(r"Before: \[(.*)\]
(.*?)
After:  \[(.*)\]")?;
    let mut string = String::new();
    input.read_to_string(&mut string)?;
    let mut samples = vec![];
    for captures in sample_regex.captures_iter(&string) {
        let before_str = &captures[1];
        let instruction_str = &captures[2];
        let after_str = &captures[3];
        let before = before_str.split(", ").map(|s| s.parse::<usize>()).collect::<Result<Vec<_>, _>>()?;
        ensure!(before.len() == 4, "Before string {} not exactly 4 ints", before_str);
        let instruction = instruction_str.split(' ').map(|s| s.parse::<usize>()).collect::<Result<Vec<_>, _>>()?;
        ensure!(instruction.len() == 4, "Instruction string {} not exactly 4 ints", before_str);
        let after = after_str.split(", ").map(|s| s.parse::<usize>()).collect::<Result<Vec<_>, _>>()?;
        ensure!(after.len() == 4, "After string {} not exactly 4 ints", after_str);
        samples.push(Sample { before, instruction, after });
    }
    let instruction_regex = Regex::new(r"\d+ \d+ \d+ \d+")?;
    let mut instructions = vec![];
    for match_ in instruction_regex.find_iter(&string).skip(samples.len()) {
        let instruction = match_.as_str().split(' ').map(|s| s.parse::<usize>()).collect::<Result<Vec<_>, _>>()?;
        instructions.push(instruction);
    }
    Ok((samples, instructions))
}

fn opcode_assignments(samples: &[Sample]) -> Option<HashMap<usize, Opcode>> {
    let mut constraints: HashMap<usize, HashSet<Opcode>> = HashMap::new();
    for sample in samples {
        let opcode = sample.instruction[0];
        let possibilities = constraints.entry(opcode).or_insert(ALL_OPS.iter().map(|&o| o).collect());
        let new_possibilities = valid_ops(&sample.before, &sample.instruction, &sample.after, possibilities.iter());
        *possibilities = possibilities.intersection(&new_possibilities.iter().map(|&o| o).collect()).map(|&o| o).collect();
    }
    let mut mapping = HashMap::new();
    let mut all_ops: Vec<_> = constraints.into_iter().collect();
    while mapping.len() != all_ops.len() {
        // Continually propagate the constraints from the most
        // constrained elements until we arrive at an answer.
        all_ops.sort_by_key(|(_o, c)| c.len());
        for i in 0..all_ops.len() {
            let (opcode, constraints) = &all_ops[i];
            if constraints.len() == 1 {
                let op = *constraints.into_iter().next().unwrap();
                mapping.insert(*opcode, op);
                for j in 0..all_ops.len() {
                    all_ops[j].1.remove(&op);
                }
                break;
            }
        }
    }
    Some(mapping)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ops() {
        assert_eq!(valid_ops(&vec![3, 2, 1, 1], &vec![9, 2, 1, 2], &vec![3, 2, 2, 1], ALL_OPS.iter()),
                   vec![AddI, MulR, SetI]);
    }

    #[test]
    fn test_parse_input() {
        let input = "Before: [3, 2, 1, 1]
9 2 1 2
After:  [3, 2, 2, 1]


Before: [1, 1, 1, 3]
5 1 3 0
After:  [3, 1, 1, 3]


1 2 3 4
1 2 4 4";
        let (samples, instructions) = parse_input(&mut input.as_bytes()).expect("Couldn't parse input");
        assert_eq!(samples, vec![Sample { before: vec![3, 2, 1, 1],
                                          instruction: vec![9, 2, 1, 2],
                                          after: vec![3, 2, 2, 1] },
                                 Sample { before: vec![1, 1, 1, 3],
                                          instruction: vec![5, 1, 3, 0],
                                          after: vec![3, 1, 1, 3] },
        ]);
        assert_eq!(instructions, vec![vec![1, 2, 3, 4],
                                      vec![1, 2, 4, 4]]);
    }
}
