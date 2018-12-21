use std::io;
use std::io::BufRead;
use failure::{Error, ensure};
use regex::Regex;

pub fn aoc16(part2: bool) -> Result<(), Error> {
    let (samples, _) = parse_input(&mut io::stdin().lock())?;
    if part2 {
    } else {
        let count = samples.iter()
            .map(|s| valid_ops(&s.before, &s.instruction, &s.after).len())
            .filter(|&n| n >= 3)
            .count();
        println!("Samples that behave like 3 or more opcodes: {}", count);
    }
    Ok(())
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Opcode {
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

struct CPU {
    registers: Vec<usize>,
}

impl CPU {
    fn new(registers: Vec<usize>) -> Self {
        Self {
            registers,
        }
    }

    fn apply_op(&mut self, op: &Opcode, arg1: usize, arg2: usize, arg3: usize) {
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

fn valid_ops(before: &Vec<usize>, instruction: &Vec<usize>, after: &Vec<usize>) -> Vec<Opcode> {
    let arg1 = instruction[1];
    let arg2 = instruction[2];
    let arg3 = instruction[3];
    let mut valid_ops = vec![];
    for op in &ALL_OPS {
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

fn parse_input(input: &mut BufRead) -> Result<(Vec<Sample>, ()), Error> {
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
    Ok((samples, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ops() {
        assert_eq!(valid_ops(&vec![3, 2, 1, 1], &vec![9, 2, 1, 2], &vec![3, 2, 2, 1]),
                   vec![AddI, MulR, SetI]);
    }

    #[test]
    fn test_parse_input() {
        let input = "Before: [3, 2, 1, 1]
9 2 1 2
After:  [3, 2, 2, 1]


Before: [1, 1, 1, 3]
5 1 3 0
After:  [3, 1, 1, 3]";
        let (samples, _) = parse_input(&mut input.as_bytes()).expect("Couldn't parse input");
        assert_eq!(samples, vec![Sample { before: vec![3, 2, 1, 1],
                                          instruction: vec![9, 2, 1, 2],
                                          after: vec![3, 2, 2, 1] },
                                 Sample { before: vec![1, 1, 1, 3],
                                          instruction: vec![5, 1, 3, 0],
                                          after: vec![3, 1, 1, 3] },
        ]);
    }
}
