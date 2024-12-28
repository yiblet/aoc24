use aoc24::{grid, parser};
use clap::Parser;
use std::io::Write;
use std::{collections::HashMap, io::Read};

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug)]
struct Robot {
    start: (isize, isize),
    velocity: (isize, isize),
}

impl Robot {
    fn step(&self, steps: usize) -> (isize, isize) {
        let steps = steps as isize;
        let (x, y) = self.start;
        let (dx, dy) = self.velocity;
        (x + dx * steps, y + dy * steps)
    }

    fn step_wrap(&self, steps: usize, bounds: (isize, isize)) -> (isize, isize) {
        wrap_pos(self.step(steps), bounds)
    }
}

fn take_robot<'a>() -> impl Fn(&'a str) -> Option<(Robot, &'a str)> {
    move |input: &str| {
        let (_, rest) = parser::take_str("p=")(input)?;
        let ((p1, _, p2), rest) = parser::take_tuple3(
            parser::take_int(),
            parser::take_str(","),
            parser::take_int(),
        )(rest)?;
        let (_, rest) = parser::take_str(" v=")(rest)?;
        let ((v1, _, v2), rest) = parser::take_tuple3(
            parser::take_int(),
            parser::take_str(","),
            parser::take_int(),
        )(rest)?;
        let start = (p1 as isize, p2 as isize);
        let velocity = (v1 as isize, v2 as isize);
        Some((Robot { start, velocity }, rest))
    }
}

fn wrap_pos(pos: (isize, isize), bounds: (isize, isize)) -> (isize, isize) {
    let (x, y) = pos;
    let (mut x, mut y) = (x % bounds.0, y % bounds.1);
    if x < 0 {
        x += bounds.0;
    }
    if y < 0 {
        y += bounds.1;
    }
    (x, y)
}

fn in_quadrant(pos: (isize, isize), bounds: (isize, isize), quadrant: usize) -> bool {
    let on_left = pos.0 < bounds.0 / 2;
    let on_right = pos.0 > bounds.0 / 2;
    let on_top = pos.1 < bounds.1 / 2;
    let on_bottom = pos.1 > bounds.1 / 2;

    let q = match (on_left, on_right, on_top, on_bottom) {
        (true, _, true, _) => 0,
        (true, _, _, true) => 1,
        (_, true, true, _) => 2,
        (_, true, _, true) => 3,
        _ => return false,
    };

    q == quadrant
}

fn parse_file(filename: &str) -> anyhow::Result<Vec<Robot>> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    let (robots, _) = parser::take_separator(take_robot(), parser::take_newline())(&string)
        .and_then(|(robots, rest)| {
            let (_, rest) = parser::take_eol()(rest)?;
            Some((robots, rest))
        })
        .ok_or_else(|| anyhow::anyhow!("could not parse"))?;
    Ok(robots)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut stdout = std::io::stdout();
    match args {
        Args::Part1 { file } => {
            let robots = parse_file(&file)?;
            let robot_count = (0usize..4)
                .map(|quadrant| {
                    robots
                        .iter()
                        .filter(|robot| {
                            let pos = robot.step(100);
                            let pos = wrap_pos(pos, (101, 103));
                            println!("{:?}", pos);
                            in_quadrant(pos, (101, 103), quadrant)
                        })
                        .count()
                })
                .collect::<Vec<_>>();

            println!("{:?}", robot_count);
            println!("{}", robot_count.iter().product::<usize>());
            Ok(())
        }

        Args::Part2 { file } => {
            let robots = parse_file(&file)?;
            let bounds = (101, 103);
            for i in 0.. {
                let positions = robots.iter().map(|robot| robot.step_wrap(i, bounds)).fold(
                    HashMap::new(),
                    |mut acc: HashMap<(isize, isize), usize>, pos| {
                        *acc.entry(pos).or_default() += 1;
                        acc
                    },
                );

                if max_line_length(&positions, bounds) < 30 {
                    continue;
                }

                let mut s = String::new();
                for y in 0..bounds.1 {
                    for x in 0..bounds.0 {
                        match positions.get(&(x, y)) {
                            Some(count) if *count == 0 => {
                                s.push('.');
                            }
                            Some(count) if *count < 10 => {
                                s.push((b'0' + *count as u8) as char);
                            }
                            Some(0) | None => {
                                s.push('.');
                            }
                            Some(_) => {
                                s.push('#');
                            }
                        }
                    }
                    s.push('\n');
                }

                write!(stdout, "\x1b[2J\n\x1b[H\n")?;
                stdout.flush()?;
                println!("iter: {}", i);
                println!("{}", s);
                stdout.flush()?;
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
            Ok(())
        }
    }
}

fn is_in_bounds(pos: (isize, isize), bounds: (isize, isize)) -> bool {
    pos.0 >= 0 && pos.0 < bounds.0 && pos.1 >= 0 && pos.1 < bounds.1
}

fn max_line_length(positions: &HashMap<(isize, isize), usize>, bounds: (isize, isize)) -> usize {
    fn check(
        positions: &HashMap<(isize, isize), usize>,
        cached: &mut HashMap<((isize, isize), grid::Direction), usize>,
        pos: (isize, isize),
        dir: grid::Direction,
        bounds: (isize, isize),
    ) -> usize {
        if let Some(v) = cached.get(&(pos, dir)) {
            return *v;
        }
        if !is_in_bounds(pos, bounds) {
            return 0;
        }
        if positions.get(&pos).is_none() {
            return 0;
        }

        let res = check(positions, cached, dir.apply(pos), dir, bounds) + 1;
        cached.insert((pos, dir), res);
        res
    }

    let mut visited = HashMap::new();
    let max_distance = positions
        .iter()
        .flat_map(|(pos, _)| {
            let visited = &mut visited;
            grid::Direction::all_directions()
                .map(move |dir| check(positions, visited, *pos, dir, bounds))
        })
        .max();

    max_distance.unwrap_or(0)
}
