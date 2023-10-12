use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use core::str::Chars;
use crate::constants;
use crate::structures::Instruction::Noop;

pub enum Type {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    Float,
    Boolean
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", match self {
            Type::U8 => "u8",
            Type::I8 => "i8",
            Type::U16 => "u16",
            Type::I16 => "i16",
            Type::U32 => "u32",
            Type::I32 => "i32",
            Type::Float => "float",
            Type::Boolean => "bool"
        })
    }
}

impl TryFrom<String> for Type {
    type Error = ();

    fn try_from(value: String) -> Result<Self, ()> {
        Ok( match value.as_str() {
            "u8" => Type::U8,
            "i8" => Type::I8,
            "u16" => Type::U16,
            "i16" => Type::I16,
            "u32" => Type::U32,
            "i32" => Type::I32,
            "float" => Type::Float,
            "bool" => Type::Boolean,
            _ => {return Err(())}
        } )
    }
}

pub enum Instruction {
    Noop,
    Load(Type),
    System(u8),
    Interrupt,
    Copy,
    Swap,
    Read(Type),
    Write(Type),
    Jump(u32),
    Branch(u32),
    Call(u32),
    Return,
    Left(Type),
    Right(Type),
    Move,
    Pointer,
    Add(Type),
    Subtract(Type),
    Multiply(Type),
    Divide(Type),
    Compare(Type),
    And(Type),
    Or(Type),
    Not(Type),
    Cast(Type, Type)
}


impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use Instruction::*;
        let (a, b, c) = match self {
            Noop => ("noop", "", ""),
            Load(ty) => ("load", ty, ""),
            System(arg) => ("system", arg, ""),
            Interrupt => ("interrupt", "", ""),
            Copy => ("copy", "", ""),
            Swap => ("swap", "", ""),
            Read(ty) => ("read", ty, ""),
            Write(ty) => ("write", ty, ""),
            Jump(ty) => ("jump", ty, ""),
            Branch(ty) => ("branch", ty, ""),
            Call(ty) => ("call", ty, ""),
            Return => ("return", "", ""),
            Left(ty) => ("left", ty, ""),
            Right(ty) => ("right", ty, ""),
            Move => ("move", "", ""),
            Pointer => ("pointer", "", ""),
            Add(ty) => ("add", ty, ""),
            Subtract(ty) => ("subtract", ty, ""),
            Multiply(ty) => ("multiply", ty, ""),
            Divide(ty) => ("divide", ty, ""),
            Compare(ty) => ("compare", ty, ""),
            And(ty) => ("and", ty, ""),
            Or(ty) => ("or", ty, ""),
            Not(ty) => ("not", ty, ""),
            Cast(a, b) => ("cast", a, b),
        };
        write!(f, "{} {} {}", a, b, c)
    }
}

#[inline]
fn get_id(index: usize, slice: &[u8]) -> Option<u32> {
    if (index > usize::MAX - 4) || (slice.len() < (index + 4)) { return None; }
    Some( u32::from_be_bytes(
        slice[index + 1 ..= index + 4].try_into().unwrap()
    ) )
}

impl Instruction {
    /// Parse an instruction from a program, given its index. Returns None if there's not enough bytes in the slice.
    //noinspection RsNonExhaustiveMatch
    fn get(slice: &[u8], index: usize) -> Option<Self> {
        use Instruction::*;

        if slice.len() < index {
            return None;
        }

        let instr_type = match slice[index] & constants::TYPE {
            0b000 => Type::U8,
            0b001 => Type::I8,
            0b010 => Type::U16,
            0b011 => Type::I16,
            0b100 => Type::U32,
            0b101 => Type::I32,
            0b110 => Type::Float,
            0b111 => Type::Boolean,
            // Due to the mask on TYPE, this cannot be reached.
            _ => unreachable!()
        };

        Some ( match (slice[index] & (!constants::TYPE)) >> 3 {
            0b00_000 => Noop,
            0b00_001 => Load(instr_type),
            0b00_010 => System(slice[index] | constants::TYPE),
            0b00_011 => Interrupt,
            0b00_100 => Copy,
            0b00_101 => Swap,
            0b00_110 => Read(instr_type),
            0b00_111 => Write(instr_type),
            0b01_000 => Jump(get_id(index, slice)?),
            0b01_001 => Branch(get_id(index, slice)?),
            0b01_010 => Call(get_id(index, slice)?),
            0b01_011 => Return,
            0b01_100 => Left(instr_type),
            0b01_101 => Right(instr_type),
            0b01_110 => Move,
            0b01_111 => Pointer,
            0b10_000 => Add(instr_type),
            0b10_001 => Subtract(instr_type),
            0b10_010 => Multiply(instr_type),
            0b10_011 => Divide(instr_type),
            0b10_100 => Compare(instr_type),
            0b10_101 => And(instr_type),
            0b10_110 => Or(instr_type),
            0b10_111 => Not(instr_type),
            0b11_000 => Cast(instr_type, Type::U8),
            0b11_001 => Cast(instr_type, Type::I8),
            0b11_010 => Cast(instr_type, Type::U16),
            0b11_011 => Cast(instr_type, Type::I16),
            0b11_100 => Cast(instr_type, Type::U32),
            0b11_101 => Cast(instr_type, Type::I32),
            0b11_110 => Cast(instr_type, Type::Float),
            0b11_111 => Cast(instr_type, Type::Boolean),
            // We matched all 32 possibilities above.
            0b100000..=0b11111111 => unreachable!()
        })
    }

    fn get_word(iter: &mut Chars) -> Option<String> {
        let mut word = String::new();
        loop {
            let char = iter.next()?;
            if char.is_ascii_whitespace() {break};
            word.push(char);
        }
        Some(word)
    }

    /// Parse a program from a string. Returns None if failed to parse.
    pub fn parse(mut program: String) -> Option<Vec<Instruction>> {
        let mut split = program.chars();
        while split.next()?.is_ascii_whitespace() {}
        let mut instructions = Vec::new();
        let mut comment_depth: usize = 0;
        while let Some(char) = split.next() {
            if char == '[' {
                comment_depth += 1;
                continue;
            }
            if char == ']' {
                if comment_depth == 0 {return None;}
                comment_depth -= 1;
                continue;
            }
            if comment_depth > 0 {continue;}
            use Instruction::*;
            let word = Self::get_word(&mut split)?;
            instructions.push(match word.as_str() {
                "noop" => Noop,
                "load" => Load(Self::get_word(&mut split)?.try_into().ok()?),
                "system" => System(Self::get_word(&mut split)?.try_into().ok()?),
                "interrupt" => Interrupt,
                "copy" => Copy,
                "swap" => Swap,
                "read" => Read(Self::get_word(&mut split)?.try_into().ok()?),
                "write" => Write(Self::get_word(&mut split)?.try_into().ok()?)
                _ => todo!()
            })
        }
        todo!()
    }
}
