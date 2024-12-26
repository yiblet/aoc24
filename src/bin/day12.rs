use std::collections::{BTreeSet, VecDeque};

use aoc24::grid;
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn get_side(edge: (isize, isize), polygon: &BTreeSet<(isize, isize)>) -> grid::Direction {
    grid::Direction::all_directions()
        .into_iter()
        .find(|dir| {
            let p2 = dir.apply(edge);
            polygon.contains(&p2)
        })
        // SAFETY: we know all polygon edges have at least one neighbor in the polygon
        .unwrap()
}

fn num_sides_from_edges(
    polygon: &BTreeSet<(isize, isize)>,
    polygon_edges: &BTreeSet<(isize, isize)>,
) -> usize {
    let mut visited = BTreeSet::new();
    let mut sides = 0;
    for pos in polygon_edges.iter().cloned() {
        if visited.contains(&pos) {
            continue;
        }
        visited.insert(pos);

        let dir = if pos.0 % 2 == 0 {
            // horizontal edge
            grid::Direction::Right
        } else {
            // vertical edge
            grid::Direction::Down
        };

        let side = get_side(pos, polygon);
        let mut next = dir.apply(dir.apply(pos));
        while polygon_edges.get(&next).is_some() {
            if get_side(next, polygon) != side {
                break;
            }
            visited.insert(next);
            next = dir.apply(dir.apply(next));
        }

        sides += 1;
    }

    sides
}

fn num_sides(polygon: &BTreeSet<(isize, isize)>) -> usize {
    if polygon.is_empty() {
        return 0;
    }

    // first scale up the grid
    let polygon: BTreeSet<(isize, isize)> = polygon
        .iter()
        .cloned()
        .map(|(x, y)| (2 * x + 1, 2 * y + 1))
        .collect();

    let polygon_edges = polygon
        .iter()
        .cloned()
        .flat_map(|pos| {
            let polygon = &polygon;
            grid::Direction::all_directions()
                .into_iter()
                .filter_map(move |dir| {
                    let potential_edge = dir.apply(pos);
                    if !polygon.contains(&dir.apply(potential_edge)) {
                        Some(potential_edge)
                    } else {
                        None
                    }
                })
        })
        .collect::<BTreeSet<_>>();

    num_sides_from_edges(&polygon, &polygon_edges)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = grid::parse_grid(&file)?;
            let partial_perimeter = grid::map(&output, |pos, chr| {
                let sames = grid::neighbors(&output, pos)
                    .filter(|(_, c2)| **c2 == *chr)
                    .count();
                4 - sames
            });

            let mut visited = grid::copy_default(&partial_perimeter);
            let mut bfs_queue = VecDeque::new();
            let mut results = Vec::new();

            for (pos, chr) in grid::iter_pos(&output) {
                let mut perimeter = 0;
                let mut area = 0;
                bfs_queue.push_back(pos);

                let get_values = |pos| {
                    let perimeter = grid::get_at(&partial_perimeter, pos)?;
                    let c = grid::get_at(&output, pos)?;
                    if *c != *chr {
                        return None;
                    }
                    Some((pos, *perimeter))
                };

                while let Some((cur, perim)) = bfs_queue.pop_front().and_then(get_values) {
                    let Some(visited) = grid::get_at_mut(&mut visited, cur) else {
                        continue;
                    };
                    if *visited {
                        continue;
                    }
                    *visited = true;
                    perimeter += perim;
                    area += 1;

                    bfs_queue.extend(grid::neighbors(&output, cur).filter_map(
                        |(cur, c2)| match *c2 == *chr {
                            true => Some(cur),
                            false => None,
                        },
                    ))
                }

                results.push((pos, perimeter, area));
            }

            let sum = results
                .iter()
                .map(|(_, perimeter, area)| perimeter * area)
                .sum::<usize>();

            println!("{}", sum);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = grid::parse_grid(&file)?;
            let polygons = get_polygons(&output);

            let sum = polygons
                .iter()
                .map(|polygon| num_sides(polygon) * polygon.len())
                .sum::<usize>();
            println!("{}", sum);
            Ok(())
        }
    }
}

fn get_polygons(output: &grid::Grid<char>) -> Vec<BTreeSet<(isize, isize)>> {
    let mut visited = grid::copy_default(output);
    let mut results = Vec::new();
    for (pos, chr) in grid::iter_pos(output) {
        if grid::get_at(&visited, pos).cloned().unwrap_or(false) {
            continue;
        }
        let mut bfs_queue = VecDeque::new();
        bfs_queue.push_back(pos);
        let mut polygon = BTreeSet::new();
        while let Some(cur) = bfs_queue.pop_front() {
            let Some(visited) = grid::get_at_mut(&mut visited, cur) else {
                continue;
            };
            if *visited {
                continue;
            }
            *visited = true;
            polygon.insert(cur);
            bfs_queue.extend(grid::neighbors(output, cur).filter_map(|(cur, c2)| {
                match *c2 == *chr {
                    true => Some(cur),
                    false => None,
                }
            }))
        }
        results.push(polygon);
    }
    results
}
