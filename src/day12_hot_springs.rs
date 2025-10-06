use crate::common::day_setup::Day;
use anyhow::Context as AnyhowContext;
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1"])
}
pub fn run(input: &str) {
    let rows: Vec<Row> = input.lines().map(|line| line.parse().unwrap()).collect();
    Row::print_arrangements(&rows, 1);
    let rows = {
        let mut rows = rows;
        rows.iter_mut().for_each(|row| row.unfold());
        rows
    };
    Row::print_arrangements(&rows, 2);
}

#[derive(Clone)]
struct Conditions(Vec<Option<bool>>);
#[derive(Clone)]
struct DamagedGroups(Vec<u8>);

#[derive(Clone)]
struct Row {
    conditions: Conditions,
    damaged_groups_rev: DamagedGroups,
}

impl Row {
    pub fn print_arrangements(values: &[Self], part: usize) {
        let mut cache = HashMap::new();
        println!(
            "part {} possible arrangements: {}",
            part,
            values
                .iter()
                .map(|row| {
                    let count = row.possible_arrangements(&mut cache);
                    log::debug!("{} - {} arrangements", row, count);
                    count
                })
                .sum::<usize>()
        );
    }
    pub fn unfold(&mut self) {
        let mut conditions = vec![None; (self.conditions.0.len() + 1) * 5 - 1];
        for (i, &condition) in self.conditions.0.iter().enumerate() {
            for j in 0..5 {
                conditions[i + (self.conditions.0.len() + 1) * j] = condition;
            }
        }
        let mut damaged = vec![0; self.damaged_groups_rev.0.len() * 5];
        for (i, &value) in self.damaged_groups_rev.0.iter().enumerate() {
            for j in 0..5 {
                damaged[i + self.damaged_groups_rev.0.len() * j] = value;
            }
        }

        self.conditions = Conditions(conditions);
        self.damaged_groups_rev = DamagedGroups(damaged);
    }
    pub fn possible_arrangements(&self, cache: &mut HashMap<CacheIndex, usize>) -> usize {
        self.possible_arrangements_recurse(0, cache)
    }
    fn no_damaged_after(&self, i: usize) -> bool {
        !self.conditions.0.iter().skip(i).any(|&c| c == Some(true))
    }
    fn possible_arrangements_recurse(
        &self,
        i: usize,
        cache: &mut HashMap<CacheIndex, usize>,
    ) -> usize {
        if i >= self.conditions.0.len() {
            return if self.damaged_groups_rev.0.is_empty() {
                log::trace!("{} is possible v1", self);
                1
            } else {
                0
            };
        }
        if self.damaged_groups_rev.0.is_empty() {
            return if self.no_damaged_after(i) {
                log::trace!("{} is possible v2", self);
                1
            } else {
                0
            };
        }
        if let Some(&result) = cache.get(&CacheIndex::new_borrowed(self, i)) {
            return result;
        }

        let mut sum = 0;
        if let Some((count, next)) = self.with_next_damaged(i) {
            sum += next.possible_arrangements_recurse(i + count, cache);
        }
        if let Some(next) = self.with_next_undamaged(i) {
            sum += next.possible_arrangements_recurse(i + 1, cache);
        }
        cache.insert(CacheIndex::new_owned(self, i), sum);
        sum
    }
    fn with_next_undamaged(&self, i: usize) -> Option<Cow<'_, Self>> {
        match self.conditions.0[i] {
            Some(true) => None,
            Some(false) => Some(Cow::Borrowed(self)),
            None => {
                let mut next = self.clone();
                next.conditions.0[i] = Some(false);
                Some(Cow::Owned(next))
            }
        }
    }
    fn with_next_damaged(&self, i: usize) -> Option<(usize, Self)> {
        let count = if let Some(count) = self.damaged_groups_rev.0.last() {
            *count as usize
        } else {
            return None;
        };

        if (i + count > self.conditions.0.len())
            || (i != 0 && self.conditions.0[i - 1] == Some(true))
            || (i + count < self.conditions.0.len() && self.conditions.0[i + count] == Some(true))
        {
            return None;
        }

        let mut next = self.clone();
        next.damaged_groups_rev.0.pop();
        for value in next.conditions.0[i..i + count].iter_mut() {
            if *value == Some(false) {
                return None;
            }
            *value = Some(true);
        }

        Some((count, next))
    }
}

#[derive(Eq, PartialEq, Hash)]
struct CacheIndex<'a>(Cow<'a, [Option<bool>]>, Cow<'a, [u8]>);

impl<'a> CacheIndex<'a> {
    pub fn new_borrowed(row: &'a Row, i: usize) -> Self {
        Self(
            Cow::Borrowed(&row.conditions.0[i.saturating_sub(1)..]),
            Cow::Borrowed(&row.damaged_groups_rev.0),
        )
    }
}

impl CacheIndex<'static> {
    pub fn new_owned(row: &Row, i: usize) -> Self {
        Self(
            Cow::Owned(row.conditions.0[i.saturating_sub(1)..].to_vec()),
            Cow::Owned(row.damaged_groups_rev.0.clone()),
        )
    }
}

impl FromStr for Row {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let conditions = Conditions(
            parts
                .next()
                .context("no condition")?
                .chars()
                .map(|c| {
                    Ok(match c {
                        '.' => Some(false),
                        '#' => Some(true),
                        '?' => None,
                        other => return Err(anyhow::anyhow!("invalid character '{}'", other)),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        let damaged_groups_rev = DamagedGroups(
            parts
                .next()
                .context("no damaged groups")?
                .split(',')
                .map(|s| s.parse())
                .rev()
                .collect::<Result<Vec<_>, _>>()?,
        );
        if let Some(next) = parts.next() {
            return Err(anyhow::anyhow!("invalid leftover part: '{}'", next));
        }

        Ok(Self {
            conditions,
            damaged_groups_rev,
        })
    }
}

impl Display for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.conditions
                .0
                .iter()
                .map(|c| match c {
                    Some(true) => '#',
                    Some(false) => '.',
                    None => '?',
                })
                .join(""),
            self.damaged_groups_rev
                .0
                .iter()
                .rev()
                .map(|val| val.to_string())
                .join(",")
        )
    }
}
