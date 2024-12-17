use std::{
    collections::{HashMap, HashSet},
    u32,
};

const INPUT: &str = include_str!("input.txt");

type Node = (usize, usize);
type Cache = HashMap<Node, Vec<Path>>;

struct Maze {
    grid: Vec<Vec<char>>,
    start: Node,
    end: Node,
    rows: usize,
    cols: usize,
}

struct SolveState {
    solvable: bool,
    // We can cache the solve result if none of the reachable nodes
    // are contained in the path we already walked.
    cacheable: bool,
}

impl Maze {
    fn new() -> Self {
        let mut start = None;
        let mut end = None;

        let grid = INPUT
            .lines()
            .enumerate()
            .map(|(i, line)| {
                line.chars()
                    .enumerate()
                    .map(|(j, c)| {
                        if c == 'S' {
                            start = Some((i, j))
                        } else if c == 'E' {
                            end = Some((i, j))
                        }
                        c
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let rows = grid.len();
        let cols = grid[0].len();

        Self {
            grid,
            start: start.unwrap(),
            end: end.unwrap(),
            rows,
            cols,
        }
    }

    fn get(&self, node: &Node) -> char {
        self.grid[node.0][node.1]
    }

    fn solvable(&self, path: &Path) -> SolveState {
        let from = path.nodes.last().unwrap();

        let mut result = HashSet::new();
        let overlaps_path = self.reachable(path, *from, true, from == &(6, 1), &mut result);

        let mut wall_count = 0;
        if from.0 > 0 && self.get(&(from.0 - 1, from.1)) == '#' {
            wall_count += 1;
        }
        if from.0 < self.rows - 1 && self.get(&(from.0 + 1, from.1)) == '#' {
            wall_count += 1;
        }
        if from.1 > 0 && self.get(&(from.0, from.1 - 1)) == '#' {
            wall_count += 1;
        }
        if from.1 < self.cols - 1 && self.get(&(from.0, from.1 + 1)) == '#' {
            wall_count += 1;
        }

        SolveState {
            solvable: result.contains(&self.end),
            cacheable: !overlaps_path && wall_count == 2,
        }
    }

    fn reachable(
        &self,
        path: &Path,
        from: Node,
        is_initial: bool,
        log: bool,
        result: &mut HashSet<Node>,
    ) -> bool {
        if self.get(&from) == '#' {
            return false;
        }

        if result.contains(&from) {
            return false;
        }

        if !is_initial && path.contains(&from) {
            return path.nodes.last().unwrap() != &from;
        }

        result.insert(from);

        let mut overlaps_path = false;

        if from.0 > 0 {
            let next = (from.0 - 1, from.1);
            if !(is_initial && path.contains(&next)) {
                if self.reachable(path, next, false, log, result) {
                    overlaps_path = true;
                }
            }
        }

        if from.0 < self.rows - 1 {
            let next = (from.0 + 1, from.1);
            if !(is_initial && path.contains(&next)) {
                if self.reachable(path, next, false, log, result) {
                    overlaps_path = true;
                }
            }
        }

        if from.1 > 0 {
            let next = (from.0, from.1 - 1);
            if !(is_initial && path.contains(&next)) {
                if self.reachable(path, next, false, log, result) {
                    overlaps_path = true;
                }
            }
        }

        if from.1 < self.cols - 1 {
            let next = (from.0, from.1 + 1);
            if !(is_initial && path.contains(&next)) {
                if self.reachable(path, next, false, log, result) {
                    overlaps_path = true;
                }
            }
        }

        overlaps_path
    }
}

#[derive(Clone, Debug)]
struct Path {
    nodes: Vec<Node>,
}

impl From<Node> for Path {
    fn from(node: Node) -> Self {
        Self { nodes: vec![node] }
    }
}

impl Path {
    fn contains(&self, node: &Node) -> bool {
        self.nodes.iter().any(|n| n == node)
    }

    fn cost(&self) -> u32 {
        let mut cost = 0;
        let mut direction = '>';

        for i in 1..self.nodes.len() {
            let a = self.nodes[i - 1];
            let b = self.nodes[i];

            match (b.0 as isize - a.0 as isize, b.1 as isize - a.1 as isize) {
                (1, 0) => {
                    if direction != 'v' {
                        cost += 1000;
                    }
                    direction = 'v';
                    cost += 1;
                }
                (-1, 0) => {
                    if direction != '^' {
                        cost += 1000;
                    }
                    direction = '^';
                    cost += 1;
                }
                (0, 1) => {
                    if direction != '>' {
                        cost += 1000;
                    }
                    direction = '>';
                    cost += 1;
                }
                (0, -1) => {
                    if direction != '<' {
                        cost += 1000;
                    }
                    direction = '<';
                    cost += 1;
                }
                _ => unreachable!(),
            }
        }

        cost
    }
}

fn _print(maze: &Maze, path: &Path) {
    println!(
        "{}\n\n===\n",
        maze.grid
            .iter()
            .enumerate()
            .map(|(i, row)| row
                .iter()
                .enumerate()
                .map(|(j, c)| if path.contains(&(i, j)) { &'X' } else { c })
                .collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn solve(maze: &Maze, path: &mut Path, cache: &mut Cache) -> Vec<Path> {
    // _print(maze, &path);

    let last = path.nodes.last().unwrap().clone();
    if maze.get(&last) == 'E' {
        return vec![path.clone()];
    }

    if let Some(paths) = cache.get(&last) {
        println!("{last:?}: CACHE HIT");
        return paths
            .clone()
            .into_iter()
            .map(|p| {
                let mut path = path.clone();
                path.nodes.extend(p.nodes);
                path
            })
            .collect::<Vec<_>>();
    }

    let mut paths = vec![];

    let solve_state = maze.solvable(path);
    if !solve_state.solvable {
        return paths;
    }

    if last.0 > 0 {
        let next = (last.0 - 1, last.1);
        if maze.get(&next) != '#' && !path.contains(&next) {
            path.nodes.push(next);
            paths.extend(solve(maze, path, cache));
            path.nodes.pop();
        }
    }

    if last.0 < maze.rows - 1 {
        let next = (last.0 + 1, last.1);
        if maze.get(&next) != '#' && !path.contains(&next) {
            path.nodes.push(next);
            paths.extend(solve(maze, path, cache));
            path.nodes.pop();
        }
    }

    if last.1 > 0 {
        let next = (last.0, last.1 - 1);
        if maze.get(&next) != '#' && !path.contains(&next) {
            path.nodes.push(next);
            paths.extend(solve(maze, path, cache));
            path.nodes.pop();
        }
    }

    if last.1 < maze.cols - 1 {
        let next = (last.0, last.1 + 1);
        if maze.get(&next) != '#' && !path.contains(&next) {
            path.nodes.push(next);
            paths.extend(solve(maze, path, cache));
            path.nodes.pop();
        }
    }

    if solve_state.cacheable {
        println!("{last:?}: CACHE INSERT");
        // _print(maze, path);
        cache.insert(
            last,
            paths
                .clone()
                .into_iter()
                .map(|mut p| {
                    while p.nodes[0] != last {
                        p.nodes.remove(0);
                    }
                    p.nodes.remove(0);
                    p
                })
                .collect::<Vec<_>>(),
        );
    }

    paths
}

fn main() {
    let maze = Maze::new();

    let mut path = Path::from(maze.start);
    let mut cache = Cache::default();
    let paths = solve(&maze, &mut path, &mut cache);
    println!(
        "{}",
        paths
            .iter()
            .fold(None, |acc, path| {
                let cost = path.cost();
                match acc {
                    None => Some(cost),
                    Some(c) if c <= cost => Some(c),
                    _ => Some(cost),
                }
            })
            .unwrap()
    );
}
