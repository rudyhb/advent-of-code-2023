use crate::common::models::grid::{GridB, GridLike};
use crate::common::models::{Direction, Point};
use crate::common::{Context, InputProvider};
use colored::Colorize;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());
    let input = context.get_input();
    let map = Map::new(&input);

    let longest_hike = map.longest_hike();
    println!("part 1: longest hike = {}", longest_hike);

    let network = Network::create(&map);
    log::debug!("network:\n{:?}", network);
    log::debug!("network intersections: {}", network.nodes.len());

    println!("part 2: longest hike = {}", solve_network(&network, &map));
}

struct Map<'a> {
    grid: GridB<'a>,
    start: Point<usize>,
    end: Point<usize>,
}

impl<'a> Map<'a> {
    pub fn new(s: &'a str) -> Self {
        let grid = GridB::new(s);
        let start = grid
            .iter_rows()
            .next()
            .unwrap()
            .1
            .find(|(_, &val)| val == b'.')
            .expect("cannot find start position")
            .0;
        let end = grid
            .iter_rows()
            .last()
            .unwrap()
            .1
            .find(|(_, &val)| val == b'.')
            .expect("cannot find start position")
            .0;
        Self { grid, start, end }
    }
    pub fn longest_hike(&self) -> usize {
        solve::<false>(self.start.clone(), Direction::Down, self)
    }
    #[allow(dead_code)]
    pub fn longest_hike_no_slippery_slopes(&self) -> usize {
        solve::<true>(self.start.clone(), Direction::Down, self)
    }
    pub fn print_with_visited(&self, visited: &HashSet<Point<usize>>) {
        println!(
            "{}",
            self.grid.display_overriding(|point| {
                if visited.contains(point) {
                    Some(format!("{}", self.grid[point] as char).bright_blue())
                } else {
                    None
                }
            })
        );
    }
}

fn solve_network(network: &Network, map: &Map) -> usize {
    let mut current = vec![NetworkState::new(network.start)];

    let mut max_cost: Option<NetworkState> = None;

    while let Some(state) = current.pop() {
        if state.current == network.end {
            log::trace!("reached end: {:?}", state);
            if max_cost
                .as_ref()
                .map(|existing| state.accrued_cost > existing.accrued_cost)
                .unwrap_or(true)
            {
                log::debug!(
                    "found a solution: {} ({} left): {:?}",
                    state.accrued_cost,
                    current.len(),
                    state.visit_order,
                );
                max_cost = Some(state);
            }
            continue;
        }
        log::trace!("state: {:?}", state);

        for next in network
            .next(state.current)
            .iter()
            .filter(|next| !state.visited.contains(&next.to))
        {
            let state = state.make_next(next);
            log::trace!("added to queue: {:?}", state);
            current.push(state);
        }
    }

    let solution = max_cost.unwrap();
    log::debug!("solution: {:?}", solution.visit_order);
    network.print_path(&solution.visit_order, map);
    solution.accrued_cost
}

#[derive(Clone, Debug)]
struct NetworkState {
    current: usize,
    visited: HashSet<usize>,
    accrued_cost: usize,
    visit_order: Vec<usize>,
}

impl NetworkState {
    pub fn new(start: usize) -> Self {
        Self {
            current: start,
            visited: HashSet::from([start]),
            accrued_cost: 0,
            visit_order: vec![start],
        }
    }
    pub fn make_next(&self, connection: &Connection) -> Self {
        let mut visited = self.visited.clone();
        visited.insert(connection.to);
        let mut visit_order = self.visit_order.clone();
        visit_order.push(connection.to);
        Self {
            current: connection.to,
            visited,
            accrued_cost: self.accrued_cost + connection.cost,
            visit_order,
        }
    }
}

#[derive(Clone, Debug)]
struct Connection {
    to: usize,
    cost: usize,
}

#[derive(Default, Clone, Debug)]
struct Node {
    connections: Vec<Connection>,
}

#[derive(Debug)]
struct Network {
    nodes: Vec<Node>,
    start: usize,
    end: usize,
    reverse_mappings: Vec<ReverseMap>,
}

type ReverseMap = (Point<usize>, HashMap<usize, Direction>);

impl Network {
    pub fn next(&self, node: usize) -> &[Connection] {
        &self.nodes[node].connections
    }
    pub fn create(map: &Map) -> Self {
        let node_coordinates: HashMap<Point<usize>, (usize, Vec<Direction>)> = map
            .grid
            .iter()
            .filter_map(|(point, &val)| {
                if val != b'#' {
                    let directions: Vec<_> = Direction::all()
                        .into_iter()
                        .filter(|&dir| {
                            map.grid
                                .move_in_direction_if(&point, dir, |(_, &val)| val != b'#')
                                .is_some()
                        })
                        .collect();
                    match directions.len() {
                        2 => None,
                        0 => unreachable!(),
                        _ => Some((point, directions)),
                    }
                } else {
                    None
                }
            })
            .enumerate()
            .map(|(i, (point, directions))| (point, (i, directions)))
            .collect();
        let mut nodes: Vec<Node> = vec![Default::default(); node_coordinates.len()];
        let end = node_coordinates
            .get(&map.end)
            .expect("could not find end coordinate")
            .0;
        let start = node_coordinates
            .get(&map.start)
            .expect("could not find start coordinate")
            .0;

        log::debug!(
            "intersections:\n{}",
            map.grid.display_overriding(|point| {
                node_coordinates
                    .get(point)
                    .map(|(i, _)| format!("{}", i).bright_blue())
            })
        );

        let mut reverse_mappings: HashMap<usize, ReverseMap> = HashMap::default();

        for (point, (index, directions)) in node_coordinates.iter() {
            let mut rev_map = (point.clone(), HashMap::new());
            let connections: Vec<Connection> = directions
                .iter()
                .map(|&dir| {
                    let mut count = 0;
                    let line = Line::<true>::follow_line(point.clone(), dir, map, |_| {
                        count += 1;
                    });
                    let other = node_coordinates
                        .get(&line.right)
                        .expect("couldn't find other line endpoint")
                        .0;

                    rev_map.1.insert(other, dir);

                    Connection {
                        to: other,
                        cost: count - 1,
                    }
                })
                .collect();
            nodes[*index] = Node { connections };
            reverse_mappings.insert(*index, rev_map);
        }

        let reverse_mappings = reverse_mappings
            .into_iter()
            .sorted_unstable_by_key(|(k, _)| *k)
            .map(|(_, v)| v)
            .collect();

        Self {
            nodes,
            start,
            end,
            reverse_mappings,
        }
    }
    pub fn print_path(&self, nodes: &[usize], map: &Map) {
        let visited: HashSet<_> = nodes
            .windows(2)
            .flat_map(|window| {
                let (left, endpoints) = &self.reverse_mappings[window[0]];
                let dir = endpoints
                    .get(&window[1])
                    .expect("cannot find direction when reconstructing path");
                let mut visited = HashSet::new();
                Line::<true>::follow_line(left.clone(), *dir, map, |m| {
                    visited.insert(m.clone());
                });
                visited
            })
            .collect();
        map.print_with_visited(&visited);
    }
}

fn solve<const ALL_DIR: bool>(start: Point<usize>, start_direction: Direction, map: &Map) -> usize {
    let max_possible = map
        .grid
        .iter()
        .filter(|(_, &val)| val as char != '#')
        .count();

    log::debug!(
        "starting a* for grid dim ({}, {}) max steps {}",
        map.grid.len_y(),
        map.grid.len_x(),
        max_possible
    );

    let mut queue: Vec<Path<ALL_DIR>> =
        vec![Path::new(Line::build_from(start, start_direction, map))];

    let mut longest_path: Option<HashSet<Point<usize>>> = None;

    while !queue.is_empty() {
        let queue_len = queue.len();
        queue.retain(|path| {
            if path.current.right == map.end {
                // end

                let members = path.get_members(map);
                log::debug!(
                    "found a solution len {}. remaining = {}",
                    members.len() - 1,
                    queue_len
                );
                if longest_path
                    .as_ref()
                    .map(|existing| existing.len() < members.len())
                    .unwrap_or(true)
                {
                    println!(
                        "found a solution len {}. remaining = {}",
                        members.len() - 1,
                        queue_len
                    );
                    longest_path = Some(members);
                }
                false
            } else {
                true
            }
        });

        queue = queue.into_iter().flat_map(|path| path.next(map)).collect();
    }

    let result = longest_path.unwrap();

    map.print_with_visited(&result);

    result.len() - 1
}

#[derive(Clone)]
struct Path<const ALL_DIR: bool> {
    current: Line<ALL_DIR>,
    lines: HashSet<Line<ALL_DIR>>,
}

impl<const ALL_DIR: bool> Path<ALL_DIR> {
    pub fn new(start: Line<ALL_DIR>) -> Self {
        Self {
            lines: HashSet::from([start.clone()]),
            current: start,
        }
    }
    pub fn get_members(&self, map: &Map) -> HashSet<Point<usize>> {
        self.lines
            .iter()
            .map(|line| line.get_members(map))
            .fold(HashSet::new(), |mut acc, next| {
                acc.extend(next);
                acc
            })
    }
    pub fn next(&self, map: &Map) -> Vec<Self> {
        log::trace!("getting next for line {:?}", self.current);

        self.current
            .next_directions(map)
            .iter()
            .filter_map(|&direction| {
                map.grid
                    .move_in_direction_if(&self.current.right, direction, |(_, &val)| val != b'#')
                    .map(|_| direction)
            })
            .map(|direction| Line::build_from(self.current.right.clone(), direction, map))
            .filter(|next| !self.lines.contains(next))
            .map(|next| self.make_next(next))
            .collect()
    }
    fn make_next(&self, next: Line<ALL_DIR>) -> Self {
        let mut this = self.clone();
        this.lines.insert(next.clone());
        this.current = next;
        this
    }
}

#[derive(Clone, Eq, Debug)]
struct Line<const ALL_DIR: bool> {
    left: Point<usize>,
    left_dir: Direction,
    right: Point<usize>,
    right_dir: Direction,
}

impl<const ALL_DIR: bool> Line<ALL_DIR> {
    pub fn build_from(start: Point<usize>, start_direction: Direction, map: &Map) -> Self {
        Self::follow_line(start, start_direction, map, |_| {})
    }
    pub fn get_members(&self, map: &Map) -> HashSet<Point<usize>> {
        let mut members = HashSet::new();
        let line = Self::follow_line(self.left.clone(), self.left_dir, map, |m| {
            members.insert(m.clone());
        });
        assert_eq!(&line, self);
        members
    }
    pub fn follow_line<F>(
        start: Point<usize>,
        start_direction: Direction,
        map: &Map,
        mut on_new_member: F,
    ) -> Self
    where
        F: FnMut(&Point<usize>),
    {
        let mut line = Self {
            right: start.move_in_direction_unchecked(start_direction),
            left: start,
            left_dir: start_direction,
            right_dir: start_direction.invert(),
        };
        on_new_member(&line.left);
        on_new_member(&line.right);

        loop {
            let directions = line.next_directions(map);
            let next: Vec<_> = directions
                .iter()
                .filter(|&&dir| dir != line.right_dir)
                .filter_map(|&direction| {
                    map.grid
                        .move_in_direction_if(&line.right, direction, |(_, &val)| val != b'#')
                        .map(|point| (direction, point))
                })
                .collect();
            if next.len() == 1 {
                let (direction, point) = next.into_iter().next().unwrap();
                on_new_member(&point);
                line.right_dir = direction.invert();
                line.right = point;
            } else {
                log::trace!("path branches out {:?}: {:?}", line, next);
                break;
            }
        }

        line
    }
    fn next_directions(&self, map: &Map) -> &[Direction] {
        if ALL_DIR {
            Direction::all_ref()
        } else {
            match map.grid[&self.right] {
                b'^' => &[Direction::Up],
                b'v' => &[Direction::Down],
                b'>' => &[Direction::Right],
                b'<' => &[Direction::Left],
                b'.' => Direction::all_ref(),
                other => panic!("invalid map character '{}'", other as char),
            }
        }
    }
}

impl<const ALL_DIR: bool> PartialEq for Line<ALL_DIR> {
    fn eq(&self, other: &Self) -> bool {
        (self.left == other.left && self.right == other.right)
            || (self.left == other.right && self.right == other.left)
    }
}

impl<const ALL_DIR: bool> Hash for Line<ALL_DIR> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.left < self.right {
            self.left.hash(state);
            self.right.hash(state);
        } else {
            self.right.hash(state);
            self.left.hash(state);
        }
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
