use anyhow::Result;
use aoc24::{
    graph::{self},
    grid::{self, Direction},
    parser,
};
use clap::Parser;
use std::{collections::BTreeMap, io::Read, rc::Rc, sync::OnceLock};

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
enum NumberPad {
    Number(u8),
    #[default]
    A,
}

fn take_numberpad<'a>() -> impl Fn(&'a str) -> Option<(NumberPad, &'a str)> {
    move |input: &str| {
        let chr = input.chars().next()?;
        match chr {
            '0'..='9' => Some((
                NumberPad::Number(chr.to_digit(10).unwrap() as u8),
                &input[chr.len_utf8()..],
            )),
            'A' => Some((NumberPad::A, &input[chr.len_utf8()..])),
            _ => None,
        }
    }
}

impl NumberPad {
    fn grid() -> &'static [[Option<Self>; 3]; 4] {
        &[
            [
                Some(NumberPad::Number(7)),
                Some(NumberPad::Number(8)),
                Some(NumberPad::Number(9)),
            ],
            [
                Some(NumberPad::Number(4)),
                Some(NumberPad::Number(5)),
                Some(NumberPad::Number(6)),
            ],
            [
                Some(NumberPad::Number(1)),
                Some(NumberPad::Number(2)),
                Some(NumberPad::Number(3)),
            ],
            [None, Some(NumberPad::Number(0)), Some(NumberPad::A)],
        ]
    }

    fn all() -> &'static [Self] {
        &[
            NumberPad::Number(9),
            NumberPad::Number(8),
            NumberPad::Number(7),
            NumberPad::Number(6),
            NumberPad::Number(5),
            NumberPad::Number(4),
            NumberPad::Number(3),
            NumberPad::Number(2),
            NumberPad::Number(1),
            NumberPad::Number(0),
            NumberPad::A,
        ]
    }

    fn shortest_paths() -> &'static ShortestPathCache<NumberPad> {
        fn init_shortest_paths() -> ShortestPathCache<NumberPad> {
            let mut graph = graph::Graph::new();
            for n1 in NumberPad::all().iter().cloned() {
                for n1n in n1.neighbors() {
                    graph::add_edge(&mut graph, n1, n1n, 1);
                }
            }

            let mut res = BTreeMap::new();
            let apsp = graph::all_pairs_shortest_paths(&graph);
            for (n1, distances) in apsp.iter() {
                for (n2, _) in distances.iter() {
                    let paths = graph::all_paths(&graph, distances, **n1, **n2);
                    res.insert((**n1, **n2), graph::paths_to_vecs(&paths, **n1, **n2));
                }
            }

            res
        }

        static NUMBERPAD_SHORTEST_PATHS: OnceLock<ShortestPathCache<NumberPad>> = OnceLock::new();

        NUMBERPAD_SHORTEST_PATHS.get_or_init(init_shortest_paths)
    }

    fn neighbors(&self) -> impl Iterator<Item = Self> {
        neighbors(Self::grid(), *self)
    }

    fn shortest_path(&self, other: &Self) -> Option<&'static Vec<Vec<Self>>> {
        let paths = Self::shortest_paths();
        let path = paths.get(&(*self, *other))?;
        Some(path)
    }
}

type ShortestPathCache<N> = BTreeMap<(N, N), Vec<Vec<N>>>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
enum ArrowPad {
    Direction(grid::Direction),
    #[default]
    A,
}
impl ArrowPad {
    fn grid() -> &'static [[Option<Self>; 3]; 2] {
        &[
            [
                None,
                Some(ArrowPad::Direction(Direction::Up)),
                Some(ArrowPad::A),
            ],
            [
                Some(ArrowPad::Direction(Direction::Left)),
                Some(ArrowPad::Direction(Direction::Down)),
                Some(ArrowPad::Direction(Direction::Right)),
            ],
        ]
    }

    fn all() -> &'static [Self] {
        &[
            ArrowPad::Direction(Direction::Up),
            ArrowPad::Direction(Direction::Left),
            ArrowPad::Direction(Direction::Right),
            ArrowPad::Direction(Direction::Down),
            ArrowPad::A,
        ]
    }

    fn shortest_paths() -> &'static ShortestPathCache<ArrowPad> {
        fn init_shortest_paths() -> ShortestPathCache<ArrowPad> {
            let mut graph = graph::Graph::new();
            for a1 in ArrowPad::all().iter().cloned() {
                for a1n in a1.neighbors() {
                    graph::add_edge(&mut graph, a1, a1n, 1);
                }
            }

            // for each node, find the shortest path to each neighbor
            let mut res = BTreeMap::new();
            let apsp = graph::all_pairs_shortest_paths(&graph);
            for (a1, distances) in apsp.iter() {
                for (a2, _) in distances.iter() {
                    let paths = graph::all_paths(&graph, distances, **a1, **a2);
                    res.insert((**a1, **a2), graph::paths_to_vecs(&paths, **a1, **a2));
                }
            }

            res
        }

        static ARROWPAD_SHORTEST_PATHS: OnceLock<ShortestPathCache<ArrowPad>> = OnceLock::new();

        ARROWPAD_SHORTEST_PATHS.get_or_init(init_shortest_paths)
    }

    fn neighbors(&self) -> impl Iterator<Item = Self> {
        neighbors(Self::grid(), *self)
    }

    fn shortest_path(&self, other: &Self) -> Option<&'static Vec<Vec<Self>>> {
        let paths = Self::shortest_paths();
        let path = paths.get(&(*self, *other))?;
        Some(path)
    }
}

fn find<const N: usize, const M: usize, T: Eq, F>(
    arr: &[[T; M]; N],
    mut pred: F,
) -> Option<(isize, isize)>
where
    F: FnMut(&T) -> bool,
{
    arr.iter().enumerate().find_map(|(i, r)| {
        r.iter().enumerate().find_map(|(j, v)| {
            if pred(v) {
                Some((i as isize, j as isize))
            } else {
                None
            }
        })
    })
}

fn neighbors<const N: usize, const M: usize, T: Eq + Clone>(
    arr: &[[Option<T>; M]; N],
    value: T,
) -> impl Iterator<Item = T> + '_ {
    find(arr, |v| *v == Some(value.clone()))
        .into_iter()
        .flat_map(move |v| {
            grid::Direction::all_directions()
                .into_iter()
                .filter_map(move |d| -> Option<T> {
                    let pos = d.apply(v);
                    if pos.0 < 0 || pos.1 < 0 {
                        return None;
                    }
                    arr.get(pos.0 as usize)?.get(pos.1 as usize).cloned()?
                })
        })
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<Vec<NumberPad>>> {
    let mut file = std::fs::File::open(filename)?;
    let mut str = String::new();
    file.read_to_string(&mut str)?;

    let take_line = parser::take_many1(take_numberpad());
    let take_lines = parser::take_separator(take_line, parser::take_newline());
    let take_input = parser::take_first(take_lines, parser::take_eol());

    let (lines, _) = take_input(str.as_str()).ok_or_else(|| anyhow::anyhow!("could not parse"))?;

    Ok(lines)
}

fn convert_numberpad_path(path: &[NumberPad]) -> anyhow::Result<Vec<ArrowPad>> {
    let mut res = vec![];
    let mut prev = None;
    for part in path.iter() {
        let part_pos = find(NumberPad::grid(), |v| *v == Some(*part))
            .ok_or_else(|| anyhow::anyhow!("could not find {:?} in grid", part))?;

        match prev {
            None => {
                prev = Some(part_pos);
            }
            Some(p) => {
                let delta = grid::vec_sub(part_pos, p);
                let dir = grid::Direction::from_delta(delta).ok_or_else(|| {
                    anyhow::anyhow!("could not find direction from {:?} to {:?}", p, part_pos)
                })?;

                res.push(ArrowPad::Direction(dir));
                prev = Some(part_pos);
            }
        }
    }

    res.push(ArrowPad::A);
    Ok(res)
}

fn convert_arrowpad_path(path: &[ArrowPad]) -> anyhow::Result<Vec<ArrowPad>> {
    let mut res = vec![];
    let mut prev = None;
    for part in path.iter() {
        let part_pos = find(ArrowPad::grid(), |v| *v == Some(*part))
            .ok_or_else(|| anyhow::anyhow!("could not find {:?} in grid", part))?;

        match prev {
            None => {
                prev = Some(part_pos);
            }
            Some(p) => {
                let delta = grid::vec_sub(part_pos, p);
                let dir = grid::Direction::from_delta(delta).ok_or_else(|| {
                    anyhow::anyhow!("could not find direction from {:?} to {:?}", p, part_pos)
                })?;

                res.push(ArrowPad::Direction(dir));
                prev = Some(part_pos);
            }
        }
    }

    res.push(ArrowPad::A);
    Ok(res)
}

type Cache = BTreeMap<(ArrowPad, ArrowPad, usize), usize>;

fn solve_path(cache: &mut Cache, path: &[NumberPad], depth: usize) -> anyhow::Result<usize> {
    let mut cur = NumberPad::default();
    let mut res = 0usize;
    for part in path.iter() {
        let Some(paths) = cur.shortest_path(part) else {
            return Err(anyhow::anyhow!(
                "no path found from {:?} to {:?}",
                cur,
                part
            ));
        };

        let path_len = paths
            .iter()
            .map(|path| convert_numberpad_path(path))
            .map(|path| -> Result<usize> {
                if depth > 0 {
                    solve_arrowpad_path(cache, &path?, depth - 1)
                } else {
                    path.map(|p| p.len())
                }
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .min()
            .ok_or_else(|| anyhow::anyhow!("no path found from {:?} to {:?}", cur, part))?;

        res += path_len;
        cur = *part;
    }
    Ok(res)
}

/// Solves the path from A to B
/// first it finds a shortest path from A to B then it converts the path int a list of
/// ArrowPad instructions that would let you move from ArrowPad A to ArrowPad B.
fn solve_arrowpad_path_simple(from: ArrowPad, to: ArrowPad) -> anyhow::Result<Vec<ArrowPad>> {
    let res = from
        .shortest_path(&to)
        .ok_or_else(|| anyhow::anyhow!("no path found from {:?} to {:?}", from, to))?
        .first()
        .ok_or_else(|| anyhow::anyhow!("no path found from {:?} to {:?}", from, to))?;

    convert_arrowpad_path(res)
}

fn solve_arrowpad_path(
    cache: &mut Cache,
    path: &[ArrowPad],
    depth: usize,
) -> anyhow::Result<usize> {
    let mut prev = ArrowPad::A;

    let mut res = 0usize;
    for part in path.iter() {
        let part_res = solve_arrowpad_path_step(cache, prev, *part, depth)?;
        res += part_res;
        prev = *part;
    }

    Ok(res)
}

fn solve_arrowpad_path_step(
    cache: &mut Cache,
    from: ArrowPad,
    to: ArrowPad,
    depth: usize,
) -> anyhow::Result<usize> {
    let idx = (from, to, depth);
    if let Some(res) = cache.get(&idx) {
        return Ok(*res);
    }

    if depth == 0 {
        let soln = Rc::new(solve_arrowpad_path_simple(from, to)?);
        cache.entry(idx).or_insert(soln.len());
        return Ok(soln.len());
    };

    let paths = from.shortest_path(&to).ok_or_else(|| {
        anyhow::anyhow!(
            "no path found from {:?} to {:?} on depth {}",
            from,
            to,
            depth
        )
    })?;

    let mut best_path = None;
    for path in paths.iter() {
        // get arrow pad path
        let path = convert_arrowpad_path(path)?;
        let res = solve_arrowpad_path(cache, &path, depth - 1)?;
        match &best_path {
            None => {
                best_path = Some(res);
            }
            Some(best) => {
                if res < *best {
                    best_path = Some(res);
                }
            }
        }
    }

    let soln =
        best_path.ok_or_else(|| anyhow::anyhow!("no path found from {:?} to {:?}", from, to))?;
    cache.entry(idx).or_insert(soln);
    Ok(soln)
}

fn convert_numberpad_path_to_number(path: &[NumberPad]) -> u64 {
    path.iter()
        .filter_map(|n| match n {
            NumberPad::Number(n) => Some(*n),
            _ => None,
        })
        .fold(0u64, |mut acc, n| {
            acc *= 10;
            acc += n as u64;
            acc
        })
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let res = parse_file(&file)?;
            let mut cache = Cache::new();

            let mut sum = 0u64;
            for path in res.iter() {
                let number = convert_numberpad_path_to_number(path);
                let arrowpads = solve_path(&mut cache, path, 2)?;
                println!("{} * {}", arrowpads, number);
                sum += number * arrowpads as u64;
            }

            println!("{}", sum);
            Ok(())
        }

        Args::Part2 { file } => {
            let res = parse_file(&file)?;
            let mut cache = Cache::new();

            let mut sum = 0u64;
            for path in res.iter() {
                let number = convert_numberpad_path_to_number(path);
                let arrowpads = solve_path(&mut cache, path, 25)?;
                println!("{} * {}", arrowpads, number);
                sum += number * arrowpads as u64;
            }

            println!("{}", sum);
            Ok(())
        }
    }
}
