use std::io;
use std::io::BufRead;
use failure::{Error, bail};
use regex::Regex;
use crate::aoc16::{CPU, Opcode, Opcode::*};

pub fn aoc19(part2: bool) -> Result<(), Error> {
    let mut cpu = parse_program(&mut io::stdin().lock())?;
    if part2 {
        cpu.cpu.registers[0] = 1;
        // Hacky bit: we know what the program *does* -- calculates
        // the sum of all factors of an integer in a certain register
        // in a super-inefficient way -- but don't really know the
        // formula for creating this number, or whether it's the same
        // formula or same register across different inputs. So this
        // may only work on my input.
        for _ in 0..(500_000 as usize) {
            cpu.step();
        }
        let factors = get_factorization(cpu.cpu.registers[1]);
        let sum: usize = factors.iter().sum();
        println!("Sum of factors: {}", sum);
    } else {
        cpu.run();
        println!("Value in register 0: {}", cpu.cpu.registers[0]);
    }
    Ok(())
}

/// Get all factors (in no particular order) of a given number.
fn get_factorization(n: usize) -> Vec<usize> {
    let mut factors = Vec::new();
    for i in 1..n {
        if i.pow(2) > n {
            // We only need to check up to (and including) sqrt(n) to
            // get all factors. This early break is me being lazy and
            // unwilling to figure out exactly the for loop condition
            // that makes sense here.
            break
        }
        if n % i == 0 {
            factors.push(i);
            if i != n / i {
                factors.push(n / i);
            }
        }
    }
    factors
}

type Instruction = (Opcode, usize, usize, usize);

#[derive(Clone)]
pub struct JumpingCPU {
    pub cpu: CPU,
    pub program: Vec<Instruction>,
    /// Index of register that acts as the instruction pointer.
    ip_index: usize,
    /// Current instruction pointer.
    pub ip: usize,
    /// Number of steps carried out so far.
    pub steps: usize,
}

impl JumpingCPU {
    /// Take a single step of execution. Returns false if the IP
    /// points outside of the program.
    pub fn step(&mut self) -> bool {
        let instruction_opt = self.program.get(self.ip);
        if let Some(instruction) = instruction_opt {
            self.cpu.registers[self.ip_index] = self.ip;
            self.cpu.apply_op(&instruction.0, instruction.1, instruction.2, instruction.3);
            self.ip = self.cpu.registers[self.ip_index];
            self.ip += 1;
            self.steps += 1;
            true
        } else {
            false
        }
    }

    /// Run the program until the IP points outside of the program.
    pub fn run(&mut self) {
        while self.step() {
        }
    }
}

pub fn parse_program(input: &mut impl BufRead) -> Result<JumpingCPU, Error> {
    let ip_set = Regex::new(r"#ip ([0-9]+)")?;
    let instruction = Regex::new(r"(.*) ([0-9]+) ([0-9]+) ([0-9]+)")?;
    let mut ip_index = 0;
    let mut program = vec![];
    for line_res in input.lines() {
        let line = line_res?;
        if let Some(captures) = ip_set.captures(&line) {
            ip_index = captures[1].parse()?;
        } else if let Some(captures) = instruction.captures(&line) {
            let opcode = match &captures[1] {
                "addr" => AddR,
                "addi" => AddI,
                "mulr" => MulR,
                "muli" => MulI,
                "banr" => BanR,
                "bani" => BanI,
                "borr" => BorR,
                "bori" => BorI,
                "setr" => SetR,
                "seti" => SetI,
                "gtir" => GtIR,
                "gtri" => GtRI,
                "gtrr" => GtRR,
                "eqir" => EqIR,
                "eqri" => EqRI,
                "eqrr" => EqRR,
                _      => bail!("Can't understand opcode {}", &captures[1]),
            };
            let arg1 = captures[2].parse()?;
            let arg2 = captures[3].parse()?;
            let arg3 = captures[4].parse()?;
            program.push((opcode, arg1, arg2, arg3));
        } else {
            bail!("Can't understand line {}", line);
        }
    }
    Ok(JumpingCPU {
        cpu: CPU::new(vec![0, 0, 0, 0, 0, 0]),
        program,
        ip: 0,
        ip_index,
        steps: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program() {
        let input_str = "#ip 0
seti 5 0 1
seti 6 0 2
addi 0 1 0
addr 1 2 3
setr 1 0 0
seti 8 0 4
seti 9 0 5";
        let program = parse_program(&mut input_str.as_bytes()).expect("Couldn't parse program");
        assert_eq!(program.ip_index, 0);
        assert_eq!(program.program, vec![
            (SetI, 5, 0, 1),
            (SetI, 6, 0, 2),
            (AddI, 0, 1, 0),
            (AddR, 1, 2, 3),
            (SetR, 1, 0, 0),
            (SetI, 8, 0, 4),
            (SetI, 9, 0, 5),
        ]);
    }

    #[test]
    fn test_jumpingcpu_run() {
        let input_str = "#ip 0
seti 5 0 1
seti 6 0 2
addi 0 1 0
addr 1 2 3
setr 1 0 0
seti 8 0 4
seti 9 0 5";
        let mut program = parse_program(&mut input_str.as_bytes()).expect("Couldn't parse program");
        program.run();
        assert_eq!(program.cpu.registers, vec![6, 5, 6, 0, 0, 9]);
    }
}
