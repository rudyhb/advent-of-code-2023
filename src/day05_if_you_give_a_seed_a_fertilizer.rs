use crate::common::day_setup::Day;
use anyhow::Context as AnyhowContext;
use std::collections::BTreeMap;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4"])
}
pub fn run(input: &str) {
    let mut blocks = input.split("\n\n");
    let seeds: Vec<i64> = blocks
        .next()
        .unwrap()
        .split(":")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .map(|word| word.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let maps = Mappings::try_from(blocks).unwrap();

    let min = seeds.iter().map(|&seed| maps.map(seed)).min().unwrap();
    println!("lowest location: {}", min);

    let seeds: Vec<_> = seeds
        .chunks_exact(2)
        .map(|chunk| {
            let start = chunk[0];
            let len = chunk[1];
            SeedRange { start, len }
        })
        .collect();
    let min = seeds
        .into_iter()
        .map(|seed| maps.map_range_to_lowest(seed))
        .min()
        .unwrap();
    println!("lowest location part 2: {}", min);
}

struct SeedRange {
    start: i64,
    len: i64,
}

#[derive(Default)]
struct SeedRanges(BTreeMap<i64, i64>);

impl SeedRanges {
    pub fn new(range: SeedRange) -> Self {
        Self(BTreeMap::from([(range.start, range.len)]))
    }
    pub fn insert(&mut self, range: SeedRange) {
        // check for largest start <= range.start
        if let Some((&start, &len)) = self.0.range(..=range.start).next_back() {
            if range.start < start + len {
                let new_len = range.len + (range.start - start);
                if new_len > len {
                    let new_range = SeedRange {
                        start,
                        len: new_len,
                    };
                    self.0.remove(&start);
                    self.insert(new_range);
                }
                return;
            }
        }

        // check for next (min) start > range.start
        if let Some((&start, &len)) = self.0.range(range.start..).next() {
            if start < range.start + range.len {
                let new_len = range.len.max(len + (start - range.start));
                let new_range = SeedRange {
                    start: range.start,
                    len: new_len,
                };
                self.0.remove(&start);
                self.insert(new_range);
                return;
            }
        }

        self.0.insert(range.start, range.len);
    }
}

struct Map {
    #[allow(dead_code)]
    source_name: String,
    destination_name: String,
    map: BTreeMap<i64, MapValue>,
}

struct MapValue {
    len: i64,
    delta: i64,
}

impl MapValue {
    pub fn new(source: i64, dest: i64, len: i64) -> Self {
        let delta = dest - source;
        Self { len, delta }
    }
}

impl Map {
    pub fn map(&self, value: i64) -> i64 {
        if let Some((&start, &MapValue { len, delta })) = self.map.range(..=value).next_back() {
            if value < start + len {
                return value + delta;
            }
        }

        value
    }
    fn map_from(&self, start: i64, len: i64) -> Vec<SeedRange> {
        // check for largest map_start <= start
        if let Some((
            &map_start,
            &MapValue {
                len: map_len,
                delta: map_delta,
            },
        )) = self.map.range(..=start).next_back()
        {
            if start < map_start + map_len {
                let map_len_diff = map_len - (start - map_start);
                if len <= map_len_diff {
                    return vec![SeedRange {
                        start: start + map_delta,
                        len,
                    }];
                }
                let mut result = vec![SeedRange {
                    start: start + map_delta,
                    len: map_len_diff,
                }];
                let extra_len = len - map_len_diff;
                let extra_start = start + map_len_diff;
                result.extend(self.map_from(extra_start, extra_len));
                return result;
            }
        }

        // check for next (min) map_start > start
        if let Some((
            &map_start,
            &MapValue {
                len: map_len,
                delta: map_delta,
            },
        )) = self.map.range(start..).next()
        {
            if map_start < start + len {
                let new_len = (len - (map_start - start)).min(map_len);
                let mut results = vec![SeedRange {
                    start: map_start + map_delta,
                    len: new_len,
                }];

                let extra_len = map_start - start;
                results.extend(self.map_from(start, extra_len));

                let len_diff = len - (map_start - start) - map_len;
                if len_diff > 0 {
                    results.extend(self.map_from(map_start + map_len, len_diff));
                }

                return results;
            }
        }

        vec![SeedRange { start, len }]
    }
    pub fn map_range(&self, range: SeedRanges) -> SeedRanges {
        range
            .0
            .into_iter()
            .flat_map(|(start, len)| self.map_from(start, len))
            .fold(SeedRanges::default(), |mut acc, next| {
                acc.insert(next);
                acc
            })
    }
}

struct Mappings([Map; 7]);

impl Mappings {
    pub fn map(&self, seed: i64) -> i64 {
        log::debug!("seed {}", seed);
        let mut value = seed;
        for map in self.0.iter() {
            value = map.map(value);
            log::debug!("{} {}", map.destination_name, value);
        }

        value
    }
    pub fn map_range_to_lowest(&self, range: SeedRange) -> i64 {
        let mut ranges = SeedRanges::new(range);
        for map in self.0.iter() {
            ranges = map.map_range(ranges);
        }
        *ranges.0.first_key_value().unwrap().0
    }
    pub fn try_from<'a>(blocks: impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        Ok(Self(
            blocks
                .map(|block| {
                    let mut lines = block.lines();
                    let mut names = lines
                        .next()
                        .unwrap()
                        .split_whitespace()
                        .next()
                        .context("empty name")?
                        .split("-");
                    let source_name = names.next().context("no source name")?.to_string();
                    let destination_name = names.nth(1).context("no destination name")?.to_string();
                    Ok(Map {
                        source_name,
                        destination_name,
                        map: lines
                            .map(|line| {
                                let mut numbers =
                                    line.split_whitespace().map(|word| word.parse::<i64>());
                                let dest = numbers.next().context("no dest number")??;
                                let source = numbers.next().context("no source number")??;
                                let len = numbers.next().context("no len number")??;
                                Ok((source, MapValue::new(source, dest, len)))
                            })
                            .collect::<anyhow::Result<BTreeMap<_, _>>>()?,
                    })
                })
                .collect::<anyhow::Result<Vec<_>>>()?
                .try_into()
                .ok()
                .context("expected 7 maps")?,
        ))
    }
}
