use crate::aoc19::parse_program;
use std::collections::HashMap;
use failure::Error;
use rayon::prelude::*;
use std::io;

/// roughly the largest value I see in the code.
/// warning: this will eat up ~20GB of RAM
const MAX_SANE_VALUE: usize = 17_000_000;

pub fn aoc21(part2: bool) -> Result<(), Error> {
    let canonical_cpu = parse_program(&mut io::stdin().lock())?;
    if part2 {
        let mut e_values = HashMap::new();
        let mut e = 0;
        let mut inner_loop_iters = 0;
        loop {
            let mut d = e | 65536;
            e = 3730679;
            loop {
                let f = d & 255;
                e = e + f;
                e = e & 16777215;
                e = e * 65899;
                e = e & 16777215;
                if d < 256 {
                    break;
                }
                inner_loop_iters += d / 256;
                d = d / 256;
            }
            println!("e: {}", e);
            if e_values.contains_key(&e) {
                break;
            }
            e_values.insert(e, inner_loop_iters);
        }
        let max = e_values.iter().max_by_key(|(_, &v)| v);
        println!("max inner loop iters: {:?}", max);
    } else {
        let mut cpus = vec![canonical_cpu; MAX_SANE_VALUE];
        for (i, cpu) in cpus.iter_mut().enumerate() {
            cpu.cpu.registers[0] = i;
        }
        loop {
            println!("iteration: {}", cpus[0].steps);
            let first = cpus.par_iter_mut()
                .enumerate()
                .map(|(i, cpu)| (i, cpu.step()))
                .find_first(|(_, not_done)| !*not_done);
            if first.is_some() {
                println!("first: {:?}", first);
                break;
            }
        }
    }
    Ok(())
}

