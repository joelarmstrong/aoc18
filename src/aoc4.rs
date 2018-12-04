use std::io;
use std::io::BufRead;
use std::collections::BTreeMap;
use failure::Error;
use regex::Regex;
use std::ops::Range;

pub fn aoc4(part2: bool) -> Result<(), Error> {
    let stdin = io::stdin();
    let sorted_string = sort_times(stdin.lock())?;
    let shifts = parse_shifts(sorted_string.as_bytes())?;
    if part2 {
        let (guard_id, minute) = find_sleepy_minute_all_guards(&shifts);
        println!("ID: {}, minute: {}, product: {}", guard_id, minute, (guard_id as usize) * minute);
    } else {
        let guard_id = find_sleepiest_guard(&shifts).ok_or_else(|| format_err!("No sleepiest guard found"))?;
        let sleepiest_minute = u64::from(find_sleepiest_minute(&shifts, guard_id));
        println!("ID: {}, minute: {}, product: {}", guard_id, sleepiest_minute, guard_id * sleepiest_minute);
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
struct GuardShift {
    id: u64,
    sleeps: Vec<Range<u8>>
}

/// Find the ID of a guard that spent the most time asleep. None if there were no guards.
fn find_sleepiest_guard(shifts: &[GuardShift]) -> Option<u64> {
    let mut guard_to_time_asleep: BTreeMap<u64, usize> = BTreeMap::new();
    for shift in shifts {
        let guard_id = shift.id;
        let time_asleep: usize = shift.sleeps.iter().map(|sleep| sleep.len()).sum();
        *guard_to_time_asleep.entry(guard_id).or_insert(0) += time_asleep;
    }
    guard_to_time_asleep.iter().max_by_key(|(_k, v)| *v).map(|(k, _v)| *k)
}

fn find_sleepy_minute_all_guards(shifts: &[GuardShift]) -> (u64, usize) {
    let mut guard_to_minutes: BTreeMap<u64, [u64; 60]> = BTreeMap::new();
    for shift in shifts {
        let minutes = guard_to_minutes.entry(shift.id).or_insert([0; 60]);
        for sleep in &shift.sleeps {
            for minute in sleep.clone() {
                minutes[minute as usize] += 1;
            }
        }
    }
    let (id, (min, _asleep)) = guard_to_minutes
        .iter()
        .map(|(id, v)| (id, v.iter().enumerate().max_by_key(|(_minute, asleep)| *asleep).unwrap()))
        .max_by_key(|(_id, (_minute, asleep))| *asleep)
        .unwrap();
    (*id, min)
}

fn find_sleepiest_minute(shifts: &[GuardShift], guard_id: u64) -> u8 {
    let mut minutes = [0; 60];
    for shift in shifts.iter().filter(|g| g.id == guard_id) {
        for sleep in &shift.sleeps {
            for minute in sleep.clone() {
                minutes[minute as usize] += 1;
            }
        }
    }
    minutes.iter().enumerate().max_by_key(|(_i, value)| *value).map(|(i, _value)| i as u8).unwrap()
}

type Time = (u32, u32, u32, u32, u32);

fn sort_times(input: impl BufRead) -> Result<String, Error> {
    let time_regex = Regex::new(r"\[(\d+)-(\d+)-(\d+) (\d+):(\d+)\]")?;
    let mut lines_and_times: Vec<(Time, String)> = vec![];
    for line_result in input.lines() {
        let line = line_result?;
        if line.is_empty() {
            continue;
        }
        let time_captures = time_regex.captures(&line).ok_or_else(|| format_err!("no time in line {}", line))?;
        let time_parts: Time = (time_captures[1].parse().unwrap(),
                                time_captures[2].parse().unwrap(),
                                time_captures[3].parse().unwrap(),
                                time_captures[4].parse().unwrap(),
                                time_captures[5].parse().unwrap());
        lines_and_times.push((time_parts, line.clone()));
    }
    lines_and_times.sort();
    let combined_string: String = lines_and_times.into_iter().map(|(_, str)| str).collect::<Vec<_>>().join("\n");
    Ok(combined_string)
}

fn parse_shifts(input: impl BufRead) -> Result<Vec<GuardShift>, Error> {
    let min_regex = Regex::new(r"\[\d+-\d+-\d+ \d+:(?P<min>\d+)\]")?;
    let begins_shift = Regex::new(r"Guard #(?P<id>\d+) begins shift$")?;

    let mut shifts = vec![];
    // Guard on the current shift
    let mut guard_id: Option<u64> = None;
    // Start of the last sleep, or None if not asleep
    let mut sleep_start: Option<u8> = None;
    // Sleeps for the current guard
    let mut cur_sleeps = vec![];
    for line_result in input.lines() {
        let line = line_result?;
        if line.is_empty() {
            // Empty line, skip
            continue
        }
        let captures = min_regex.captures(&line).ok_or_else(|| format_err!("no time in line {}", line))?;
        let minute: u8 = captures[1].parse()?;
        if let Some(captures) = begins_shift.captures(&line) {
            if let Some(prev_guard_id) = guard_id {
                // Finish previous guard's shift.
                shifts.push(GuardShift { id: prev_guard_id, sleeps: cur_sleeps });
                cur_sleeps = vec![];
            }
            guard_id = Some(captures[1].parse()?);
        } else if line.ends_with("falls asleep") {
            sleep_start = Some(minute);
        } else if line.ends_with("wakes up") {
            let start = sleep_start.ok_or_else(|| format_err!("Guard {:?} woke up before falling asleep", guard_id))?;
            cur_sleeps.push(start..minute);
            sleep_start = None;
        } else {
            bail!("Line {} didn't match any expected pattern", line);
        }
    }

    if let Some(prev_guard_id) = guard_id {
        // Finish last guard's shift.
        shifts.push(GuardShift { id: prev_guard_id, sleeps: cur_sleeps });
    }

    Ok(shifts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use rand::prelude::SliceRandom;
    use rand::thread_rng;

    /// This function allows us to assert that a Result is
    /// Ok(expected) without requiring PartialEq on the Error type.
    fn assert_result_ok<T: Debug + PartialEq>(r: Result<T, Error>, expected: T) {
        match r {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("got Err: {}, local backtrace: {}", e, e.backtrace()),
        }
    }

    const TIME_STRING: &'static str = "
[1518-11-01 00:00] Guard #10 begins shift
[1518-11-01 00:05] falls asleep
[1518-11-01 00:25] wakes up
[1518-11-01 00:30] falls asleep
[1518-11-01 00:55] wakes up
[1518-11-01 23:58] Guard #99 begins shift
[1518-11-02 00:40] falls asleep
[1518-11-02 00:50] wakes up
[1518-11-03 00:05] Guard #10 begins shift
[1518-11-03 00:24] falls asleep
[1518-11-03 00:29] wakes up
[1518-11-04 00:02] Guard #99 begins shift
[1518-11-04 00:36] falls asleep
[1518-11-04 00:46] wakes up
[1518-11-05 00:03] Guard #99 begins shift
[1518-11-05 00:45] falls asleep
[1518-11-05 00:55] wakes up
";

    #[test]
    fn test_parse_shifts() {
        assert_result_ok(parse_shifts(TIME_STRING.as_bytes()), vec![
            GuardShift {
                id: 10,
                sleeps: vec![(5..25), (30..55)],
            },
            GuardShift {
                id: 99,
                sleeps: vec![(40..50)],
            },
            GuardShift {
                id: 10,
                sleeps: vec![(24..29)],
            },
            GuardShift {
                id: 99,
                sleeps: vec![(36..46)],
            },
            GuardShift {
                id: 99,
                sleeps: vec![(45..55)],
            },
        ])
    }

    #[test]
    fn test_find_sleepiest_minute() {
        let shifts = parse_shifts(TIME_STRING.as_bytes()).expect("Can't parse shifts");
        assert_eq!(find_sleepiest_minute(&shifts, 10), 24);
    }

    #[test]
    fn test_find_sleepiest_guard() {
        let shifts = parse_shifts(TIME_STRING.as_bytes()).expect("Can't parse shifts");
        assert_eq!(find_sleepiest_guard(&shifts), Some(10));
    }

    #[test]
    fn test_sort_times() {
        let mut lines: Vec<_> = TIME_STRING.lines().collect();
        let mut rng = thread_rng();
        lines.shuffle(&mut rng);
        let shuffled_string: String = lines.join("\n");
        let sorted = sort_times(shuffled_string.as_bytes());
        assert_eq!(sorted.unwrap(), TIME_STRING.trim());
    }

    #[test]
    fn test_find_sleepy_minute_all_guards() {
        let shifts = parse_shifts(TIME_STRING.as_bytes()).expect("Can't parse shifts");
        let (guard_id, minute) = find_sleepy_minute_all_guards(&shifts);
        assert_eq!(guard_id, 99);
        assert_eq!(minute, 45);
    }
}
