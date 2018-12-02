use std::io;
use std::io::BufRead;
use failure::Error;

#[derive(Debug, Fail)]
enum Aoc1Error {
    #[fail(display = "can't parse change: {}", string)]
    ParseChangeError {
        string: String,
    },
}

pub fn aoc1(part2: bool) {
    // This let binding is needed for stdin to live long enough
    let stdin = io::stdin();
    println!("Sum: {}", sum_up_changes(&mut stdin.lock()).expect("Error encountered: "));
}

/// Parse a change like "+1" or "-1".
fn parse_change(line: &String) -> Result<i64, Error> {
    let change: i64 = line.parse()
        .or_else(|_| Err(Aoc1Error::ParseChangeError {string: line.to_string()}))?;
    Ok(change)
}

/// Implements part 1.
fn sum_up_changes(input: &mut impl BufRead) -> Result<i64, Error> {
    let changes: Vec<i64> = input.lines()
        .flat_map(|l_res| l_res.map(|l| parse_change(&l)))
        .collect::<Result<Vec<i64>, Error>>()?;
    Ok(changes.iter().sum())
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
}
