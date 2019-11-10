use std::collections::HashMap;
use std::collections::HashSet;
use std::cmp::Reverse;
use std::io;
use std::io::BufRead;
use std::u32;
use regex::Regex;
use rayon::prelude::*;
use failure::{Error, format_err};

pub fn aoc24(part2: bool) -> Result<(), Error> {
    let mut armies = parse_armies(&mut io::stdin().lock())?;
    if part2 {
        let (boost, unit_count, _) = (0..u32::MAX).into_par_iter().map(|boost| {
            let mut new_armies = armies.clone();
            println!("Attempting boost {}", boost);
            if !fight(&mut new_armies, boost) {
                println!("Inconclusive");
                return (boost, 0, false);
            }
            let unit_count = new_armies.iter().map(|g| g.units).sum::<u32>();
            if new_armies.iter().filter(|g| g.team == ImmuneSystem).count() != 0 {
                // Immune system won.
                return (boost, unit_count, true);
            }
            (boost, unit_count, false)
        }).find_first(|(_, _, b)| *b).unwrap();
        println!("Immune system won with boost {}, final number of units: {}", boost, unit_count);
    } else {
        fight(&mut armies, 0);
        println!("Armies: {:?}, final number of units: {}", armies, armies.iter().map(|g| g.units).sum::<u32>());
    }
    Ok(())
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
enum Army {
    Infection,
    ImmuneSystem,
}

use Army::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Group {
    hp: u32,
    units: u32,
    damage: u32,
    attack_type: String,
    initiative: u32,
    weaknesses: Vec<String>,
    immunities: Vec<String>,
    team: Army,
}

impl Group {
    fn effective_power(&self, boost: u32) -> u32 {
        let mut damage = self.damage;
        if self.team == ImmuneSystem {
            damage += boost;
        }
        self.units * damage
    }

    fn damage_to(&self, other: &Group, boost: u32) -> u32 {
        if other.weaknesses.contains(&self.attack_type) {
            self.effective_power(boost) * 2
        } else if other.immunities.contains(&self.attack_type) {
            0
        } else {
            self.effective_power(boost)
        }
    }

    fn select_target<'a>(&'a self, my_index: usize, groups: &'a[Group], targeted_by: &mut HashMap<usize, usize>, boost: u32) {
        let target_opt = groups.iter()
            .enumerate()
            .filter(|(_, g)| g.team != self.team)
            .filter(|(i, _)| !targeted_by.contains_key(i))
            .filter(|(_, g)| self.damage_to(g, boost) > 0)
            .max_by_key(|(_, g)| (self.damage_to(g, boost), g.effective_power(boost), g.initiative));
        if let Some((index, _)) = target_opt {
            targeted_by.insert(index, my_index);
        }
    }

    fn take_damage(&mut self, damage: u32) {
        self.units = self.units.saturating_sub(damage / self.hp);
    }
}

fn select_targets<'a>(groups: &'a[Group], boost: u32) -> HashMap<usize, usize> {
    let mut targeted_by = HashMap::new();
    let mut sorted: Vec<_> = groups.iter().enumerate().collect();
    sorted.sort_by_key(|(_, g)| Reverse((g.effective_power(boost), g.initiative)));
    for (original_index, group) in sorted.iter() {
        group.select_target(*original_index, groups, &mut targeted_by, boost);
    }
    targeted_by
}

fn attack_round(groups: &mut Vec<Group>, boost: u32) {
    groups.sort_by_key(|g| Reverse(g.initiative));
    let targeted_by = select_targets(&groups, boost);
    // FIXME: invert targets since it happened to be the wrong way around after all.
    let targets: HashMap<_, _> = targeted_by.iter().map(|(k, v)| (v, k)).collect();
    for i in 0..groups.len() {
        if let Some(attacked_index) = targets.get(&i) {
            let damage = groups[i].damage_to(&groups[**attacked_index], boost);
            groups[**attacked_index].take_damage(damage);
        }
    }
    groups.retain(|g| g.units > 0);
}

fn fight(groups: &mut Vec<Group>, boost: u32) -> bool {
    loop {
        let teams: HashSet<Army> = groups.iter().map(|g| g.team).collect();
        if teams.len() < 2 {
            return true;
        }
        let total_units_before: u32 = groups.iter().map(|g| g.units).sum();
        attack_round(groups, boost);
        let total_units_after = groups.iter().map(|g| g.units).sum();
        if total_units_before == total_units_after {
            // Stuck.
            return false;
        }
    }
}

fn parse_group(input_line: &str, team: Army) -> Result<Group, Error> {
    let line_regex = Regex::new(r"(?P<units>[0-9]+) units each with (?P<hp>[0-9]+) hit points\s*\(?(?P<weak_immune>.*?)\)?\s*with an attack that does (?P<damage>[0-9]+) (?P<damage_type>.*) damage at initiative (?P<initiative>[0-9]+)")?;
    let captures = line_regex.captures(input_line).ok_or_else(|| format_err!("Can't understand line {}", input_line))?;
    let units: u32 = captures["units"].parse()?;
    let hp: u32 = captures["hp"].parse()?;

    let weak_immune_regex = Regex::new(r"(weak to (?P<weak>.*))|(immune to (?P<immune>.*))")?;
    let mut weaknesses = vec![];
    let mut immunities = vec![];
    if let Some(m) = captures.name("weak_immune") {
        for part in m.as_str().split("; ") {
            if part.trim().is_empty() {
                continue;
            }
            let sub_caps = weak_immune_regex.captures(part).ok_or_else(|| format_err!("Can't understand weakness/immunity string {}", part))?;
            if let Some(m) = sub_caps.name("weak") {
                weaknesses.extend(m.as_str().split(",").map(|s| s.trim().to_string()));
            } else if let Some(m) = sub_caps.name("immune") {
                immunities.extend(m.as_str().split(",").map(|s| s.trim().to_string()));
            }
        }
    }
    let damage: u32 = captures["damage"].parse()?;
    let attack_type: String = captures["damage_type"].to_string();
    let initiative: u32 = captures["initiative"].parse()?;
    Ok(Group {
        units,
        hp,
        weaknesses,
        immunities,
        damage,
        attack_type,
        initiative,
        team
    })
}

fn parse_armies(read: &mut impl BufRead) -> Result<Vec<Group>, Error> {
    let mut team = None;
    let mut ret = vec![];
    for line_res in read.lines() {
        let line = line_res?.trim().to_string();
        if line.is_empty() {
            // Blank line.
            continue
        }
        match line.as_str() {
            "Immune System:" => team = Some(ImmuneSystem),
            "Infection:" => team = Some(Infection),
            _ => ret.push(parse_group(&line, team.ok_or_else(|| format_err!("Got a group before a team"))?)?),
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref GROUPS: Vec<Group> = {
vec![
            Group {
                units: 17,
                hp: 5390,
                weaknesses: vec!["radiation".to_string(), "bludgeoning".to_string()],
                immunities: vec![],
                damage: 4507,
                attack_type: "fire".to_string(),
                initiative: 2,
                team: ImmuneSystem,
            },
            Group {
                units: 989,
                hp: 1274,
                weaknesses: vec!["bludgeoning".to_string(), "slashing".to_string()],
                immunities: vec!["fire".to_string()],
                damage: 25,
                attack_type: "slashing".to_string(),
                initiative: 3,
                team: ImmuneSystem,
            },
            Group {
                units: 801,
                hp: 4706,
                weaknesses: vec!["radiation".to_string()],
                immunities: vec![],
                damage: 116,
                attack_type: "bludgeoning".to_string(),
                initiative: 1,
                team: Infection,
            },
            Group {
                units: 4485,
                hp: 2961,
                weaknesses: vec!["fire".to_string(), "cold".to_string()],
                immunities: vec!["radiation".to_string()],
                damage: 12,
                attack_type: "slashing".to_string(),
                initiative: 4,
                team: Infection,
            },
        ]
        };
    }

    #[test]
    fn test_parse_group() -> Result<(), Error> {
        let input_str = "18 units each with 729 hit points (weak to fire; immune to cold, slashing) with an attack that does 8 radiation damage at initiative 10";
        let group = parse_group(input_str, Infection)?;
        assert_eq!(group, Group {
            units: 18,
            hp: 729,
            weaknesses: vec!["fire".to_string()],
            immunities: vec!["cold".to_string(), "slashing".to_string()],
            damage: 8,
            attack_type: "radiation".to_string(),
            initiative: 10,
            team: Infection,
        });
        Ok(())
    }

    #[test]
    fn test_effective_power() -> Result<(), Error> {
        let input_str = "18 units each with 729 hit points (weak to fire; immune to cold, slashing) with an attack that does 8 radiation damage at initiative 10";
        let group = parse_group(input_str, ImmuneSystem)?;
        assert_eq!(group.effective_power(0), 144);
        assert_eq!(group.effective_power(1570), 28404);
        Ok(())
    }

    #[test]
    fn test_parse_armies() -> Result<(), Error> {
        let input_str = "
Immune System:
17 units each with 5390 hit points (weak to radiation, bludgeoning) with an attack that does 4507 fire damage at initiative 2
989 units each with 1274 hit points (immune to fire; weak to bludgeoning, slashing) with an attack that does 25 slashing damage at initiative 3

Infection:
801 units each with 4706 hit points (weak to radiation) with an attack that does 116 bludgeoning damage at initiative 1
4485 units each with 2961 hit points (immune to radiation; weak to fire, cold) with an attack that does 12 slashing damage at initiative 4
";
        let groups = parse_armies(&mut input_str.as_bytes())?;
        assert_eq!(groups, *GROUPS);
        Ok(())
    }

    #[test]
    fn test_select_targets() {
        let targeted_by = select_targets(&GROUPS, 0);
        assert_eq!(targeted_by[&3], 0);
        assert_eq!(targeted_by[&2], 1);
        assert_eq!(targeted_by[&0], 2);
        assert_eq!(targeted_by[&1], 3);
    }

    #[test]
    fn test_group_take_damage() {
        let mut group_1 = GROUPS[0].clone();
        let mut group_2 = GROUPS[1].clone();
        let mut group_3 = GROUPS[2].clone();
        let mut group_4 = GROUPS[3].clone();

        group_2.take_damage(group_4.damage_to(&group_2, 0));
        assert_eq!(group_2.units, 905);

        group_3.take_damage(group_2.damage_to(&group_3, 0));
        assert_eq!(group_3.units, 797);

        group_4.take_damage(group_1.damage_to(&group_4, 0));
        assert_eq!(group_4.units, 4434);

        group_1.take_damage(group_3.damage_to(&group_1, 0));
        assert_eq!(group_1.units, 0);
    }

    #[test]
    fn test_attack_round() {
        let mut groups = GROUPS.clone();
        attack_round(&mut groups, 0);
        assert_eq!(groups, vec![
            Group {
                units: 4434,
                hp: 2961,
                weaknesses: vec!["fire".to_string(), "cold".to_string()],
                immunities: vec!["radiation".to_string()],
                damage: 12,
                attack_type: "slashing".to_string(),
                initiative: 4,
                team: Infection,
            },
            Group {
                units: 905,
                hp: 1274,
                weaknesses: vec!["bludgeoning".to_string(), "slashing".to_string()],
                immunities: vec!["fire".to_string()],
                damage: 25,
                attack_type: "slashing".to_string(),
                initiative: 3,
                team: ImmuneSystem,
            },
            Group {
                units: 797,
                hp: 4706,
                weaknesses: vec!["radiation".to_string()],
                immunities: vec![],
                damage: 116,
                attack_type: "bludgeoning".to_string(),
                initiative: 1,
                team: Infection,
            },
        ]);
    }

    #[test]
    fn test_fight() {
        let mut groups = GROUPS.clone();
        fight(&mut groups, 0);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.iter().map(|g| g.units).sum::<u32>(), 5216);
    }

    #[test]
    fn test_fight_with_boost() {
        let mut groups = GROUPS.clone();
        fight(&mut groups, 1570);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups.iter().map(|g| g.units).sum::<u32>(), 51);
    }
}
