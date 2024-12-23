#!/usr/bin/env python3

template = """\
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 { file: String },
    /// Day 1 part 2
    Part2 { file: String },
}

fn parse_file(filename: &str) -> anyhow::Result<String> {
    let mut file = std::fs::File::open(filename)?;
    todo!()
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file } => {
            let output = parse_file(&file)?;

            todo!();
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;

            todo!();
            Ok(())
        }
    }
}
"""

def main():
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("day", type=int)
    args = parser.parse_args()

    with open("Cargo.toml", "a") as f:
        f.write(f"\n[[bin]]\nname = \"day{args.day:02}\"\npath = \"src/bin/day{args.day:02}.rs\"\n")

    with open(f"src/bin/day{args.day:02}.rs", "w") as f:
        f.write(template)

if __name__ == "__main__":
    main()
