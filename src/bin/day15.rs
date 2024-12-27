use std::{
    collections::{btree_map, BTreeMap, BTreeSet},
    io::Read,
};

use aoc24::{grid, parser};
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 {
        file: String,
        #[arg(long, action = clap::ArgAction::SetTrue)]
        debug: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    Empty,
    Box,
    Wall,
    Robot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry2 {
    Empty,
    LBox,
    RBox,
    Wall,
    Robot,
}

fn take_entry<'a>() -> impl Fn(&'a str) -> Option<(Entry, &'a str)> {
    move |input: &str| {
        let (entry, rest) = parser::take_any_char()(input)?;
        match entry {
            '.' => Some((Entry::Empty, rest)),
            '#' => Some((Entry::Wall, rest)),
            '@' => Some((Entry::Robot, rest)),
            'O' => Some((Entry::Box, rest)),
            _ => None,
        }
    }
}

fn take_entries<'a>() -> impl Fn(&'a str) -> Option<(Vec<Entry>, &'a str)> {
    parser::take_many1(take_entry())
}

fn take_moves<'a>() -> impl Fn(&'a str) -> Option<(Vec<grid::Direction>, &'a str)> {
    let take_move = move |input: &'a str| {
        let (chr, rest) = parser::take_any("<>^v")(input)?;
        let dir = match chr {
            '<' => grid::Direction::Left,
            '>' => grid::Direction::Right,
            '^' => grid::Direction::Up,
            'v' => grid::Direction::Down,
            _ => None?,
        };
        Some((dir, rest))
    };

    parser::take_many1(take_move)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedResult {
    entries: Vec<Vec<Entry>>,
    moves: Vec<grid::Direction>,
}

fn move_to(
    entries: &mut grid::Grid<Entry>,
    pos: grid::Index,
    dir: grid::Direction,
) -> Option<grid::Index> {
    let p2 = dir.apply(pos);
    match grid::get_at(entries, pos) {
        Some(Entry::Empty) => Some(p2),
        Some(entry @ Entry::Box) | Some(entry @ Entry::Robot) => {
            let entry = *entry;
            if move_to(entries, p2, dir).is_some() {
                // SAFETY: we know that the position in the next pos is within the bounds
                // already through the recursive call
                *grid::get_at_mut(entries, p2).unwrap() = entry;
                // SAFETY: we know that the position in the current pos is within the bounds
                // already
                *grid::get_at_mut(entries, pos).unwrap() = Entry::Empty;
                Some(p2)
            } else {
                None
            }
        }
        None | Some(Entry::Wall) => None,
    }
}

fn move_to3(
    entries: &mut grid::Grid<Entry2>,
    pos: grid::Index,
    dir: grid::Direction,
) -> Option<grid::Index> {
    // algorithm: first we mark all the positions that have to be moved in unison.
    let mut visited = BTreeSet::new();
    let mut to_move = Vec::new();
    let mut to_visit = BTreeSet::from([pos]);
    while let Some(cur) = to_visit.pop_first() {
        let entry = *grid::get_at(entries, cur)?;
        if !visited.contains(&cur) {
            visited.insert(cur);
        } else {
            continue;
        }
        match entry {
            Entry2::Empty => {}
            Entry2::Wall => return None,
            Entry2::Robot => {
                let pos2 = dir.apply(cur);
                to_move.push((cur, entry));
                to_visit.insert(pos2);
            }
            Entry2::LBox | Entry2::RBox if dir.is_horizontal() => {
                let pos2 = dir.apply(cur);
                to_move.push((cur, entry));
                to_visit.insert(pos2);
            }
            Entry2::LBox => {
                let rbox_pos = grid::Direction::Right.apply(cur);
                if !(visited.contains(&rbox_pos) || to_visit.contains(&rbox_pos)) {
                    to_visit.insert(rbox_pos);
                }
                let pos2 = dir.apply(cur);
                to_move.push((cur, entry));
                to_visit.insert(pos2);
            }
            Entry2::RBox => {
                let lbox_pos = grid::Direction::Left.apply(cur);
                if !(visited.contains(&lbox_pos) || to_visit.contains(&lbox_pos)) {
                    to_visit.insert(lbox_pos);
                }
                let pos2 = dir.apply(cur);
                to_move.push((cur, entry));
                to_visit.insert(pos2);
            }
        }
    }

    let mut positions_set = BTreeSet::new();
    for (cur, entry) in to_move.iter().cloned() {
        let pos2 = dir.apply(cur);
        // SAFETY: we know that the position in the next pos is within the bounds
        // already through the check at line 108
        if !positions_set.contains(&cur) {
            *grid::get_at_mut(entries, cur).unwrap() = Entry2::Empty;
        }
        *grid::get_at_mut(entries, pos2).unwrap() = entry;
        positions_set.insert(pos2);
    }

    Some(dir.apply(pos))
}

fn move_to2(
    entries: &mut grid::Grid<Entry2>,
    pos: grid::Index,
    dir: grid::Direction,
    from: Option<grid::Direction>,
) -> Option<grid::Index> {
    // GUARD: move_to2 returns Some if the position is opened up meaning that the entry
    // at that position did move to the new position.
    //
    // This also means that if move_to2 returns Some, then we know that the entry at
    // p2 is now empty.
    fn simple_move(
        entries: &mut Vec<Vec<Entry2>>,
        pos: (isize, isize),
        dir: grid::Direction,
        entry: Entry2,
    ) -> Option<(isize, isize)> {
        let p2 = dir.apply(pos);
        let new_from = Some(dir.invert());
        move_to2(entries, p2, dir, new_from)?;
        // SAFETY: we know that the position in the next pos is within the bounds
        // already through the recursive call
        *grid::get_at_mut(entries, p2).unwrap() = entry;
        // SAFETY: we know that the position in the current pos is within the bounds
        // already
        *grid::get_at_mut(entries, pos).unwrap() = Entry2::Empty;
        Some(p2)
    }

    let p2 = dir.apply(pos);
    match grid::get_at(entries, pos) {
        // Entry2:::Empty works because empty spaces can move anywhere
        Some(Entry2::Empty) => Some(p2),
        // Walls and boundaries can't move
        Some(Entry2::Wall) | None => None,

        // robots can move but only if item in front of them is an empty space.
        Some(Entry2::Robot) => simple_move(entries, pos, dir, Entry2::Robot),
        // LBox and RBox can move horizontally (and move simply when horizontal)
        Some(entry @ Entry2::LBox) | Some(entry @ Entry2::RBox) if dir.is_horizontal() => {
            simple_move(entries, pos, dir, *entry)
        }
        Some(Entry2::LBox) => {
            println!("{:?}", from);
            let rbox_pos = grid::Direction::Right.apply(pos);
            assert!(matches!(
                grid::get_at(entries, rbox_pos),
                Some(Entry2::RBox)
            ));
            // if we are moving vertically, we need to move the right box first
            // the match statement is to make sure through recursive calls we don't
            // move the box twice.
            //
            // We track from which direction we came from so that we aren't already
            // moving the box based on the RBox.
            if !matches!(from, Some(grid::Direction::Right)) {
                let lbox_from = Some(grid::Direction::Right.invert());
                move_to2(entries, rbox_pos, dir, lbox_from)?;
            }

            simple_move(entries, pos, dir, Entry2::LBox)
        }
        Some(Entry2::RBox) => {
            let lbox_pos = grid::Direction::Left.apply(pos);
            assert!(matches!(
                grid::get_at(entries, lbox_pos),
                Some(Entry2::LBox)
            ));
            // if we are moving vertically, we need to move the left box first
            // the match statement is to make sure through recursive calls we don't
            // move the box twice.
            //
            // We track from which direction we came from so that we aren't already
            // moving the box based on the LBox.
            if !matches!(from, Some(grid::Direction::Left)) {
                let rbox_from = Some(grid::Direction::Left.invert());
                move_to2(entries, lbox_pos, dir, rbox_from)?;
            }

            simple_move(entries, pos, dir, Entry2::RBox)
        }
    }
}

fn take_result<'a>() -> impl Fn(&'a str) -> Option<(ParsedResult, &'a str)> {
    move |input: &'a str| {
        let (entries, rest) =
            parser::take_separator(take_entries(), parser::take_newline())(input)?;
        let (_, rest) = parser::take_newline()(rest)?;
        let (moves, rest) = parser::take_separator(take_moves(), parser::take_newline())(rest)?;
        let (_, rest) = parser::take_eol()(rest)?;
        Some((
            ParsedResult {
                entries,
                moves: moves.into_iter().flatten().collect(),
            },
            rest,
        ))
    }
}

fn convert_to_part2(entries: &grid::Grid<Entry>) -> grid::Grid<Entry2> {
    let mut res = Vec::with_capacity(entries.len());
    for row in entries.iter() {
        let mut row2 = Vec::with_capacity(row.len() * 2);
        for entry in row {
            match entry {
                Entry::Empty => row2.extend([Entry2::Empty, Entry2::Empty]),
                Entry::Box => row2.extend([Entry2::LBox, Entry2::RBox]),
                Entry::Wall => row2.extend([Entry2::Wall, Entry2::Wall]),
                Entry::Robot => row2.extend([Entry2::Robot, Entry2::Empty]),
            }
        }

        res.push(row2);
    }

    res
}

fn parse_file(filename: &str) -> anyhow::Result<ParsedResult> {
    let mut file = std::fs::File::open(filename)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let (result, rest) = take_result()(&s).ok_or_else(|| anyhow::anyhow!("could not parse"))?;
    if !rest.is_empty() {
        Err(anyhow::anyhow!("could not parse"))?;
    }
    Ok(result)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let mut output = parse_file(&file)?;
            let start = grid::iter_pos(&output.entries)
                .find(|(_, chr)| **chr == Entry::Robot)
                .ok_or_else(|| anyhow::anyhow!("could not find robot"))?
                .0;

            let mut cur = start;
            for dir in output.moves {
                if let Some(p) = move_to(&mut output.entries, cur, dir) {
                    cur = p;
                }
            }

            let res = grid::iter_pos(&output.entries)
                .filter_map(|(pos, chr)| {
                    if *chr == Entry::Box {
                        Some(100 * pos.0 + pos.1)
                    } else {
                        None
                    }
                })
                .sum::<isize>();

            println!("{}", res);
            Ok(())
        }

        Args::Part2 { file, debug } => {
            let ParsedResult { entries, moves } = parse_file(&file)?;
            let mut entries = convert_to_part2(&entries);
            let start = grid::iter_pos(&entries)
                .find(|(_, chr)| **chr == Entry2::Robot)
                .ok_or_else(|| anyhow::anyhow!("could not find robot"))?
                .0;

            let mut cur = start;
            if debug {
                println!("{}", entries2_to_string(&entries));
            }
            for dir in moves {
                if let Some(p) = move_to3(&mut entries, cur, dir) {
                    cur = p;
                }
                if debug {
                    println!("{}", entries2_to_string(&entries));
                }
            }

            let res = grid::iter_pos(&entries)
                .filter_map(|(pos, chr)| {
                    if *chr == Entry2::LBox {
                        Some(100 * pos.0 + pos.1)
                    } else {
                        None
                    }
                })
                .sum::<isize>();

            if debug {
                let s = entries2_to_string(&entries);
                println!("{}", s);
            }
            println!("{}", res);
            Ok(())
        }
    }
}

fn entries2_to_string(entries: &grid::Grid<Entry2>) -> String {
    let mut s = String::new();
    for row in entries.iter() {
        for entry in row {
            match entry {
                Entry2::Empty => s.push('.'),
                Entry2::Wall => s.push('#'),
                Entry2::Robot => s.push('@'),
                Entry2::LBox => s.push('['),
                Entry2::RBox => s.push(']'),
            }
        }
        s.push('\n');
    }
    s
}
