use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};

use aoc24::parser;
use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Args {
    /// Day 1 part 1
    Part1 {
        file: String,
        #[arg(long, action = clap::ArgAction::SetTrue)]
        debug: bool,
        #[arg(long, short = 'a')]
        set_a: Option<i64>,
    },
    /// Day 1 part 2
    Part2 {
        file: String,
    },

    Mappings {
        file: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Register {
    A,
    B,
    C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Combo {
    Value(u8),
    Register(Register),
    Seven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Adv(Combo),
    Bxl(Literal),
    Bst(Combo),
    Jnz(Literal),
    Bxc(Literal),
    Out(Combo),
    Bdv(Combo),
    Cdv(Combo),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Literal(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegisterState {
    a: i64,
    b: i64,
    c: i64,
    ip: usize,
}

// The adv instruction (opcode 0) performs division. The numerator is the value in the A register. The denominator is found by raising 2 to the power of the instruction's combo operand. (So, an operand of 2 would divide A by 4 (2^2); an operand of 5 would divide A by 2^B.) The result of the division operation is truncated to an integer and then written to the A register.

// The bxl instruction (opcode 1) calculates the bitwise XOR of register B and the instruction's literal operand, then stores the result in register B.

// The bst instruction (opcode 2) calculates the value of its combo operand modulo 8 (thereby keeping only its lowest 3 bits), then writes that value to the B register.

// The jnz instruction (opcode 3) does nothing if the A register is 0. However, if the A register is not zero, it jumps by setting the instruction pointer to the value of its literal operand; if this instruction jumps, the instruction pointer is not increased by 2 after this instruction.

// The bxc instruction (opcode 4) calculates the bitwise XOR of register B and register C, then stores the result in register B. (For legacy reasons, this instruction reads an operand but ignores it.)

// The out instruction (opcode 5) calculates the value of its combo operand modulo 8, then outputs that value. (If a program outputs multiple values, they are separated by commas.)

// The bdv instruction (opcode 6) works exactly like the adv instruction except that the result is stored in the B register. (The numerator is still read from the A register.)

// The cdv instruction (opcode 7) works exactly like the adv instruction except that the result is stored in the C register. (The numerator is still read from the A register.)

impl RegisterState {
    fn apply(&self, instr: &Instruction) -> (Self, Option<i64>) {
        match instr {
            Instruction::Adv(combo) => self.adv(combo),
            Instruction::Bxl(literal) => self.bxl(literal),
            Instruction::Bst(combo) => self.bst(combo),
            Instruction::Jnz(literal) => self.jnz(literal),
            Instruction::Bxc(literal) => self.bxc(literal),
            Instruction::Out(combo) => self.out(combo),
            Instruction::Bdv(combo) => self.bdv(combo),
            Instruction::Cdv(combo) => self.cdv(combo),
        }
    }

    fn combo(&self, combo: &Combo) -> i64 {
        match combo {
            Combo::Value(v) => *v as i64,
            Combo::Register(r) => match r {
                Register::A => self.a,
                Register::B => self.b,
                Register::C => self.c,
            },
            Combo::Seven => unimplemented!(),
        }
    }

    fn literal(&self, literal: &Literal) -> i64 {
        literal.0 as i64
    }

    // division of a by 2^combo set to a.
    fn adv(&self, combo: &Combo) -> (Self, Option<i64>) {
        self.dv(combo, Register::A)
    }

    // bitwise xor of b and literal then set to b.
    fn bxl(&self, literal: &Literal) -> (Self, Option<i64>) {
        let mut res = *self;
        res.b ^= self.literal(literal);
        res.ip += 2;
        (res, None)
    }

    // bst instruction: set b to combo mod 8.
    fn bst(&self, combo: &Combo) -> (Self, Option<i64>) {
        let mut res = *self;
        res.b = self.combo(combo) % 8;
        res.ip += 2;
        (res, None)
    }

    // jnz instruction: if a is 0, jump to literal.
    fn jnz(&self, literal: &Literal) -> (Self, Option<i64>) {
        let mut res = *self;
        if res.a != 0 {
            res.ip = self.literal(literal) as usize;
        } else {
            res.ip += 2;
        }
        (res, None)
    }

    // bxc instruction: set b to b xor c.
    fn bxc(&self, _literal: &Literal) -> (Self, Option<i64>) {
        let mut res = *self;
        res.b ^= res.c;
        res.ip += 2;
        (res, None)
    }

    // out instruction: output combo mod 8.
    fn out(&self, combo: &Combo) -> (Self, Option<i64>) {
        let mut res = *self;
        let v = self.combo(combo) % 8;
        res.ip += 2;
        (res, Some(v))
    }

    // bdv instruction: set b to a / 2^combo.
    fn bdv(&self, combo: &Combo) -> (Self, Option<i64>) {
        self.dv(combo, Register::B)
    }

    // cdv instruction: set c to a / 2^combo.
    fn cdv(&self, combo: &Combo) -> (Self, Option<i64>) {
        self.dv(combo, Register::C)
    }

    fn dv(&self, combo: &Combo, reg: Register) -> (Self, Option<i64>) {
        let mut res = *self;
        let num = self.a;
        let comb = self.combo(combo);
        let val = if comb < 0 {
            num * 2i64.pow(comb.unsigned_abs() as u32)
        } else {
            num / 2i64.pow(comb as u32)
        };

        match reg {
            Register::A => {
                res.a = val;
            }
            Register::B => {
                res.b = val;
            }
            Register::C => {
                res.c = val;
            }
        }

        res.ip += 2;
        (res, None)
    }
}

fn take_register<'a>(register: Register) -> impl Fn(&'a str) -> Option<(i64, &'a str)> {
    move |input: &str| {
        let (_, rest) = parser::take_str("Register ")(input)?;
        let (_, rest) = parser::take_char(match register {
            Register::A => 'A',
            Register::B => 'B',
            Register::C => 'C',
        })(rest)?;
        let (_, rest) = parser::take_str(": ")(rest)?;
        let (a, rest) = parser::take_int()(rest)?;
        Some((a, rest))
    }
}

fn take_register_state<'a>() -> impl Fn(&'a str) -> Option<(RegisterState, &'a str)> {
    move |input: &str| {
        let (a, rest) = take_register(Register::A)(input)?;
        let (_, rest) = parser::take_newline()(rest)?;
        let (b, rest) = take_register(Register::B)(rest)?;
        let (_, rest) = parser::take_newline()(rest)?;
        let (c, rest) = take_register(Register::C)(rest)?;
        let (_, rest) = parser::take_newline()(rest)?;
        Some((RegisterState { a, b, c, ip: 0 }, rest))
    }
}

fn convert_instruction(input: &[i64; 2]) -> Option<Instruction> {
    let op = input[0];
    let operand = input[1];

    let instr = match op {
        0 => Instruction::Adv(convert_combo(operand as u8)?),
        1 => Instruction::Bxl(convert_literal(operand as u8)?),
        2 => Instruction::Bst(convert_combo(operand as u8)?),
        3 => Instruction::Jnz(convert_literal(operand as u8)?),
        4 => Instruction::Bxc(convert_literal(operand as u8)?),
        5 => Instruction::Out(convert_combo(operand as u8)?),
        6 => Instruction::Bdv(convert_combo(operand as u8)?),
        7 => Instruction::Cdv(convert_combo(operand as u8)?),
        _ => return None,
    };

    Some(instr)
}

fn convert_literal(a: u8) -> Option<Literal> {
    let res = match a {
        v @ 0..=7 => Literal(v),
        _ => return None,
    };
    Some(res)
}

fn convert_combo(a: u8) -> Option<Combo> {
    let res = match a {
        0 => Combo::Value(0),
        1 => Combo::Value(1),
        2 => Combo::Value(2),
        3 => Combo::Value(3),
        4 => Combo::Register(Register::A),
        5 => Combo::Register(Register::B),
        6 => Combo::Register(Register::C),
        7 => Combo::Seven,
        _ => return None,
    };
    Some(res)
}

#[derive(Debug)]
struct Program {
    states: RegisterState,
    instructions: Vec<i64>,
}

fn take_program<'a>() -> impl Fn(&'a str) -> Option<(Program, &'a str)> {
    move |input: &str| {
        let (registers, rest) = take_register_state()(input)?;
        let (_, rest) = parser::take_newline()(rest)?;
        let (_, rest) = parser::take_str("Program: ")(rest)?;
        let (instructions, rest) =
            parser::take_separator(parser::take_int(), parser::take_char(','))(rest)?;
        let (_, rest) = parser::take_newline()(rest)?;
        let (_, rest) = parser::take_eol()(rest)?;
        Some((
            Program {
                states: registers,
                instructions,
            },
            rest,
        ))
    }
}

fn parse_file(filename: &str) -> anyhow::Result<Program> {
    let mut file = std::fs::File::open(filename)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    let (program, _) = take_program()(&string).ok_or_else(|| anyhow::anyhow!("could not parse"))?;
    Ok(program)
}

fn run_program(mut state: RegisterState, instructions: &[i64]) -> anyhow::Result<Vec<i64>> {
    let mut res = Vec::new();
    while state.ip + 1 < instructions.len() {
        let instr = instructions[state.ip];
        let instr2 = instructions[state.ip + 1];
        let instr = convert_instruction(&[instr, instr2])
            .ok_or_else(|| anyhow::anyhow!("could not convert {:?} {:?}", instr, instr2))?;
        let (new_state, output) = state.apply(&instr);
        state = new_state;
        if let Some(v) = output {
            res.push(v);
        }
    }
    Ok(res)
}

fn run_program_iter(
    mut state: RegisterState,
    instructions: &[i64],
    debug: bool,
) -> impl Iterator<Item = anyhow::Result<i64>> + '_ {
    let mut errored = false;
    std::iter::from_fn(move || {
        if errored {
            return None;
        }

        while state.ip + 1 < instructions.len() {
            let instr = instructions[state.ip];
            let instr2 = instructions[state.ip + 1];
            let instr = match convert_instruction(&[instr, instr2])
                .ok_or_else(|| anyhow::anyhow!("could not convert {:?} {:?}", instr, instr2))
            {
                Ok(instr) => instr,
                Err(e) => {
                    errored = true;
                    return Some(Err(e));
                }
            };

            let (new_state, output) = state.apply(&instr);
            if debug {
                println!(
                    "instr = {:?}, state = {:?}, new_state = {:?}, output = {:?}",
                    instr, state, new_state, output
                );
            }
            state = new_state;
            if let Some(v) = output {
                return Some(Ok(v));
            }
        }

        None
    })
}

fn run_program_with_a(
    a: i64,
    instructions: &[i64],
) -> impl Iterator<Item = anyhow::Result<i64>> + '_ {
    run_program_iter(
        RegisterState {
            a,
            b: 0,
            c: 0,
            ip: 0,
        },
        instructions,
        false,
    )
}

fn run_part2(
    instructions: &[i64],
    idx: usize,
    cur: i64,
    solutions: &mut BTreeSet<i64>,
) -> anyhow::Result<()> {
    for i in 0..8 {
        let a = cur << 3 | i;
        let output = run_program_with_a(a, instructions).collect::<Result<Vec<_>, _>>()?;
        if output == instructions[instructions.len() - 1 - idx..] {
            if idx + 1 == instructions.len() {
                solutions.insert(a);
            } else {
                run_part2(instructions, idx + 1, a, solutions)?;
            }
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args {
        Args::Part1 { file, debug, set_a } => {
            let output = parse_file(&file)?;
            let mut state = output.states;
            if let Some(a) = set_a {
                state.a = a;
            }
            let res = run_program_iter(state, &output.instructions, debug)
                .collect::<anyhow::Result<Vec<_>>>()?;

            println!(
                "{}",
                res.into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            Ok(())
        }

        Args::Part2 { file } => {
            let output = parse_file(&file)?;
            let mut solutions = BTreeSet::new();
            run_part2(&output.instructions, 0, 0, &mut solutions)?;
            println!("{:?}", solutions.first());
            Ok(())
        }

        Args::Mappings { file } => {
            let output = parse_file(&file)?;

            // expected sequence of outputs to counts of internal states;
            let mut results = BTreeMap::<[i64; 3], [i64; 8]>::new();
            for i in 1024..=1024 * 1024 {
                let mut state = output.states;
                state.a = i;
                let mut cursor = [-1i64; 3];

                let mut cur = i;
                let mut program = run_program_iter(state, &output.instructions, false);
                while let Some(Ok(v)) = program.next() {
                    cursor[0] = cursor[1];
                    cursor[1] = cursor[2];
                    cursor[2] = v;

                    results.entry(cursor).or_default()[cur.unsigned_abs() as usize % 8] += 1;
                    cur /= 8;
                }
            }

            println!("{:?}", results);

            let mut mle = 0i64;
            let mut cursor = [-1i64; 3];
            let mut buf = Vec::new();

            for expected in output.instructions.iter().rev() {
                buf.clear();

                cursor[0] = cursor[1];
                cursor[1] = cursor[2];
                cursor[2] = *expected;

                let iter = results
                    .get(&cursor)
                    .ok_or_else(|| anyhow::anyhow!("could not find expected {:?}", cursor))?
                    .iter()
                    .enumerate();

                for (i, v) in iter {
                    buf.push((v, i));
                }

                buf.sort();

                println!("{:?}", buf);
                let idx = buf[7].1;
                mle += mle * 8 + idx as i64;
            }

            println!("mle: {}", mle);

            let mut state = output.states;
            state.a = mle;
            let outputs = run_program(state, &output.instructions)?
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            println!("got: {}", outputs);
            println!(
                "expected: {}",
                output
                    .instructions
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            Ok(())
        }
    }
}
