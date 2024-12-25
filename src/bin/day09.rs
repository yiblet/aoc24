use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};

use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Id(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Size(usize);

fn files_and_spaces(string: &str) -> (BTreeMap<usize, Id>, BTreeSet<usize>) {
    let mut files = BTreeMap::new();
    let mut spaces = BTreeSet::new();
    let mut pos = 0;

    for (i, c) in string.chars().filter(|c| c.is_ascii_digit()).enumerate() {
        if i % 2 == 0 {
            let id = i / 2;
            for _ in 0..c.to_digit(10).unwrap() {
                files.insert(pos, Id(id));
                pos += 1;
            }
        } else {
            for _ in 0..c.to_digit(10).unwrap() {
                spaces.insert(pos);
                pos += 1;
            }
        }
    }
    (files, spaces)
}

type FilesAndSpacesResult = (BTreeMap<usize, (Id, Size)>, BTreeSet<(usize, Size)>);

fn files_and_spaces2(string: &str) -> FilesAndSpacesResult {
    let mut files = BTreeMap::new();
    let mut spaces = BTreeSet::new();
    let mut pos = 0usize;

    for (i, c) in string.chars().filter(|c| c.is_ascii_digit()).enumerate() {
        let size = Size(c.to_digit(10).unwrap() as usize);
        if i % 2 == 0 {
            let id = i / 2;
            files.insert(pos, (Id(id), size));
            pos += size.0;
        } else {
            spaces.insert((pos, size));
            pos += size.0;
        }
    }
    (files, spaces)
}

fn insert_space2(spaces: &mut BTreeSet<(usize, Size)>, pos: usize, size: Size) {
    spaces.insert((pos, size));
    merge_spaces(spaces, pos, size);
}

fn merge_spaces(spaces: &mut BTreeSet<(usize, Size)>, pos: usize, size: Size) {
    match spaces.range(..(pos, size)).next_back().cloned() {
        Some((p2, s2)) if p2 + s2.0 == pos => {
            spaces.remove(&(p2, s2));
            spaces.remove(&(pos, size));
            spaces.insert((p2, Size(s2.0 + size.0)));
            merge_spaces(spaces, p2, Size(size.0 + s2.0));
        }
        _ => match spaces.range((pos, size)..).nth(1).cloned() {
            Some((p2, s2)) if pos + size.0 == p2 => {
                spaces.remove(&(pos, size));
                spaces.remove(&(p2, s2));
                spaces.insert((pos, Size(size.0 + s2.0)));
                merge_spaces(spaces, pos, Size(size.0 + s2.0));
            }
            _ => {}
        },
    }
}

fn parse_file(filename: &str) -> anyhow::Result<String> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;
            let (mut files, mut spaces) = files_and_spaces(&output);
            while files
                .last_key_value()
                .zip(spaces.first())
                .map_or(false, |(f, s)| s < f.0)
            {
                if let Some((f, id)) = files.pop_last() {
                    if let Some(s) = spaces.pop_first() {
                        files.insert(s, id);
                        spaces.insert(f);
                    }
                }
            }

            let res = files.into_iter().map(|(pos, id)| pos * id.0).sum::<usize>();
            println!("{}", res);
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let (files, mut spaces) = files_and_spaces2(&output);
            let mut final_files = BTreeMap::new();

            let mut vec = files
                .into_iter()
                .map(|(p, (i, s))| (i, p, s))
                .collect::<Vec<_>>();
            vec.sort();
            vec.reverse();

            for (id, pos, size) in vec.into_iter() {
                match spaces.range(..).find(|s| s.1 >= size) {
                    Some(space) if space.0 < pos => {
                        let space = *space;
                        final_files.insert(space.0, (id, size));
                        spaces.remove(&space);
                        if space.1 > size {
                            let new_size = Size(space.1 .0 - size.0);
                            let new_pos = space.0 + size.0;
                            insert_space2(&mut spaces, new_pos, new_size);
                            insert_space2(&mut spaces, pos, size);
                        } else {
                            insert_space2(&mut spaces, pos, size);
                        }
                    }
                    _ => {
                        final_files.insert(pos, (id, size));
                    }
                }
            }

            let res = final_files
                .into_iter()
                .map(|(pos, (id, size))| {
                    let pos_sum = pos * size.0 + (size.0 - 1) * size.0 / 2;
                    pos_sum * id.0
                })
                .sum::<usize>();

            println!("{}", res);
            Ok(())
        }
    }
}
