use std::io;
use std::collections::BTreeSet;
use std::io::BufRead;
use failure::Error;

#[derive(Debug, Fail)]
enum Aoc1Error {
    #[fail(display = "can't parse change: {}", string)]
    ParseChangeError {
        string: String,
    },
    #[fail(display = "didn't find any duplicates")]
    NoDuplicatesFoundError,
}

pub fn aoc1(part2: bool) -> Result<(), Error> {
    // This let binding is needed for stdin to live long enough
    let stdin = io::stdin();
    if part2 {
        println!("First dup: {}", first_duplicate_freq(&mut stdin.lock())?);
    } else {
        println!("Sum: {}", sum_up_changes(&mut stdin.lock())?);
    }
    Ok(())
}

/// Parse a change like "+1" or "-1".
fn parse_change(line: &String) -> Result<i64, Error> {
    let change: i64 = line.parse()
        .or_else(|_| Err(Aoc1Error::ParseChangeError {string: line.to_string()}))?;
    Ok(change)
}

/// Parse multiple changes from a file.
fn parse_changes(input: &mut impl BufRead) -> Result<Vec<i64>, Error> {
    input.lines()
        .flat_map(|l_res| l_res.map(|l| parse_change(&l)))
        .collect::<Result<Vec<i64>, Error>>()
}

/// Implements part 1.
fn sum_up_changes(input: &mut impl BufRead) -> Result<i64, Error> {
    let changes: Vec<i64> = parse_changes(input)?;
    Ok(changes.iter().sum())
}

/// Implements part 2.
fn first_duplicate_freq(input: &mut impl BufRead) -> Result<i64, Error> {
    let changes: Vec<i64> = parse_changes(input)?;
    let mut seen = BTreeSet::new();
    let mut cur_freq = 0;
    for change in changes.iter().cycle() {
        println!("change: {} cur_freq: {}", change, cur_freq);
        if seen.contains(&cur_freq) {
            return Ok(cur_freq);
        }
        seen.insert(cur_freq);
        cur_freq += change;
    }

    // If we got here, there weren't any dups. Someone fucked up.
    Err(Error::from(Aoc1Error::NoDuplicatesFoundError))
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
            Err(e) => panic!("got error result {}", e),
        }
    }

    #[test]
    fn test_sum_up_changes() {
        let mut input = "+1\n+1\n+1".as_bytes();
        assert_result_ok(sum_up_changes(&mut input), 3);

        input = "+1\n+1\n-2\n".as_bytes();
        assert_result_ok(sum_up_changes(&mut input), 0);

        input = "-1\n-2\n-3\n".as_bytes();
        assert_result_ok(sum_up_changes(&mut input), -6);
    }

    #[test]
    fn test_first_duplicate_freq() {
        let mut input = "+1\n-1".as_bytes();
        assert_result_ok(first_duplicate_freq(&mut input), 0);

        input = "+3\n+3\n+4\n-2\n-4".as_bytes();
        assert_result_ok(first_duplicate_freq(&mut input), 10);

        input = "-6\n+3\n+8\n+5\n-6".as_bytes();
        assert_result_ok(first_duplicate_freq(&mut input), 5);

        input = "+7\n+7\n-2\n-7\n-4".as_bytes();
        assert_result_ok(first_duplicate_freq(&mut input), 14);
    }
}
