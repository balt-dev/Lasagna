pub use structures::{Type, Instruction, parse};

mod structures {
    use std::collections::BTreeMap;
    use std::fmt::{Display, Debug, Formatter};
    use std::str::Chars;
    use crate::constants;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    #[repr(u8)]
    pub enum Type {
        U8 = 0b000,
        I8 = 0b001,
        U16 = 0b010,
        I16 = 0b011,
        U32 = 0b100,
        I32 = 0b101,
        Float = 0b110,
        Boolean = 0b111
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

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Instruction {
        Noop,
        Load(Type, u32),
        System(u8),
        Interrupt,
        Copy,
        Swap,
        Read(Type),
        Write(Type),
        Jump(u32),
        Branch(Type, u32),
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
        Cast(Type, Type),
        ShiftLeft,
        ShiftRight,
        RotLeft,
        RotRight,
        Xor(Type),
        Break
    }

    impl Display for Instruction {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            use Instruction::*;
            match self {
                Noop =>
                    write!(f, "noop"),
                Load(ty, val) => { write!(f, "load ")?; match ty {
                    Type::U8 => write!(f, "{}", *val as u8),
                    Type::I8 => write!(f, "{}", *val as i8),
                    Type::U16 => write!(f, "{}", *val as u16),
                    Type::I16 => write!(f, "{}", *val as i16),
                    Type::U32 => write!(f, "{}", *val),
                    Type::I32 => write!(f, "{}", *val as i32),
                    Type::Float => write!(f, "{}", f32::from_bits(*val)),
                    Type::Boolean => write!(f, "{}", *val as u8 != 0)
                }},
                System(arg) =>
                    write!(f, "system {}", arg),
                Interrupt =>
                    write!(f, "interrupt"),
                Copy =>
                    write!(f, "copy"),
                Swap =>
                    write!(f, "swap"),
                Read(ty) =>
                    write!(f, "read {}", ty),
                Write(ty) =>
                    write!(f, "write {}", ty),
                Jump(num) =>
                    write!(f, "jump __{:08X}", num),
                Branch(ty, num) =>
                    write!(f, "branch {} __{:08X}", ty, num),
                Call(num) =>
                    write!(f, "call __{:08X}", num),
                Return =>
                    write!(f, "return"),
                Left(ty) =>
                    write!(f, "left {}", ty),
                Right(ty) =>
                    write!(f, "right {}", ty),
                Move =>
                    write!(f, "move"),
                Pointer =>
                    write!(f, "pointer"),
                Add(ty) =>
                    write!(f, "add {}", ty),
                Subtract(ty) =>
                    write!(f, "subtract {}", ty),
                Multiply(ty) =>
                    write!(f, "multiply {}", ty),
                Divide(ty) =>
                    write!(f, "divide {}", ty),
                Compare(ty) =>
                    write!(f, "compare {}", ty),
                And(ty) =>
                    write!(f, "and {}", ty),
                Or(ty) =>
                    write!(f, "or {}", ty),
                Not(ty) =>
                    write!(f, "not {}", ty),
                Cast(a, b) =>
                    write!(f, "cast {} {}", a, b),
                ShiftLeft =>
                    write!(f, "shiftleft"),
                ShiftRight =>
                    write!(f, "shiftright"),
                RotLeft =>
                    write!(f, "rotleft"),
                RotRight =>
                    write!(f, "rotright"),
                Xor(ty) =>
                    write!(f, "xor {}", ty),
                Break =>
                    write!(f, "break"),
            }
        }
    }

    #[inline]
    fn get_id(index: usize, slice: &[u8]) -> Option<u32> {
        if (index > usize::MAX - 4) || (slice.len() < (index + 4)) { return None; }
        Some( u32::from_be_bytes(
            slice[index + 1 ..= index + 4].try_into().unwrap()
        ) )
    }

    #[derive(Debug)]
    enum ParseInstr {
        Instr(Instruction),
        Jump(String),
        Branch(Type, String),
        Call(String)
    }

    impl Instruction {
        /// Parse an instruction from a program, given its index. Returns None if there's not enough bytes in the slice.
        //noinspection RsNonExhaustiveMatch
        pub fn get(slice: &[u8], index: usize) -> Option<Self> {
            use Instruction::*;

            if slice.len() < index {
                return None;
            }

            let (instr_type, instr_size) = match slice[index] & constants::TYPE {
                0b000 => (Type::U8, 1),
                0b001 => (Type::I8, 1),
                0b010 => (Type::U16, 2),
                0b011 => (Type::I16, 2),
                0b100 => (Type::U32, 4),
                0b101 => (Type::I32, 4),
                0b110 => (Type::Float, 4),
                0b111 => (Type::Boolean, 1),
                // Due to the mask on TYPE, this cannot be reached.
                0b1000..=0b11111111 => unreachable!()
            };

            let part = (slice[index] & (!constants::TYPE)) >> 3;

            Some ( match part {
                0b00_000 => Noop,
                0b00_001 => {
                    if usize::MAX - index <= instr_size {
                        return None;
                    }
                    let value = &slice[index + 1 ..= index + instr_size];
                    let value = match instr_type {
                        Type::U8 | Type::I8 | Type::Boolean => value[0] as u32,
                        Type::U16 | Type::I16 =>
                            u16::from_be_bytes(
                                value[0..=1].try_into().unwrap()
                            ) as u32,
                        Type::U32 | Type::I32 | Type::Float =>
                            u32::from_be_bytes(
                                value[0..=3].try_into().unwrap()
                            )
                    };
                    Load(instr_type, value)
                },
                0b00_010 => System(slice[index] | constants::TYPE),
                0b00_011 => Interrupt,
                0b00_100 => Copy,
                0b00_101 => Swap,
                0b00_110 => Read(instr_type),
                0b00_111 => Write(instr_type),
                0b01_000 => Jump(get_id(index, slice)?),
                0b01_001 => Branch(instr_type, get_id(index, slice)?),
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
                part if (
                    part & constants::GROUP == 0b11000000
                ) && (
                    (part & constants::INDEX) >> 3 == (instr_type as u8)
                ) => match part {
                    0b11_000 => ShiftLeft,
                    0b11_001 => ShiftRight,
                    0b11_010 => RotLeft,
                    0b11_011 => RotRight,
                    0b11_111 => Break,
                    _ => Xor(instr_type),
                }
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
    }

    fn get_word(iter: &mut Chars) -> Option<String> {
        let mut word = String::new();
        let mut iter = iter.peekable();
        while iter.peek()?.is_ascii_whitespace() {
            iter.next();
        }
        loop {
            let char = iter.next();
            if char.is_none() {break}
            let char = char.unwrap();
            if char.is_ascii_whitespace() {break}
            word.push(char);
        }
        Some(word)
    }

    /// Parse a program from a string. Returns None if failed to parse.
    pub fn parse(program: String) -> Option<Vec<Instruction>> {
        let split = program.trim().chars();

        // Remove comments
        let mut comment_depth: usize = 0;
        let mut no_comments = String::new();

        for char in split {
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
            no_comments.push(char);
        }

        // Create instructions

        let mut labels: BTreeMap<String, u32> = BTreeMap::new();

        let mut instructions = Vec::new();
        use Instruction::*;

        for line in no_comments.lines() {
            let line = line.trim();
            if line.is_empty() {continue;}

            let mut split = line.chars();

            use ParseInstr::Instr;
            use Type::*;
            let word = get_word(&mut split)?;

            let parse_instr = match word.as_str() {
                "noop" =>
                    Instr(Noop),
                "load" => {
                    let value = get_word(&mut split)?;
                    Instr(if value == "true" {
                        Load(Boolean, 1)
                    } else if value == "false" {
                        Load(Boolean, 0)
                    } else if let Some(string_num) = value.strip_suffix("_u8") {
                        let num: u8 = string_num.parse().ok()?;
                        Load(U8, num as u32)
                    } else if let Some(string_num) = value.strip_suffix("_i8") {
                        let num: i8 = string_num.parse().ok()?;
                        Load(I8, num as u32)
                    } else if let Some(string_num) = value.strip_suffix("_u16") {
                        let num: u16 = string_num.parse().ok()?;
                        Load(U16, num as u32)
                    } else if let Some(string_num) = value.strip_suffix("_i16") {
                        let num: i16 = string_num.parse().ok()?;
                        Load(I16, num as u32)
                    } else if let Some(string_num) = value.strip_suffix("_u32") {
                        let num: u32 = string_num.parse().ok()?;
                        Load(U32, num)
                    } else if let Some(string_num) = value.strip_suffix("_i32") {
                        let num: i32 = string_num.parse().ok()?;
                        Load(I32, num as u32)
                    } else {
                        let num: f32 = value.parse().ok()?;
                        Load(Float,
                             u32::from_be_bytes(num.to_be_bytes())
                        )
                    })
                },
                "system" =>
                    Instr(System(get_word(&mut split)?.parse().ok()?)),
                "interrupt" =>
                    Instr(Interrupt),
                "copy" =>
                    Instr(Copy),
                "swap" =>
                    Instr(Swap),
                "read" =>
                    Instr(Read(get_word(&mut split)?.try_into().ok()?)),
                "write" =>
                    Instr(Write(get_word(&mut split)?.try_into().ok()?)),
                "label" => {
                    labels.insert(split.collect::<String>(), instructions.len() as u32);
                    continue;
                },
                "jump" =>
                    ParseInstr::Jump(split.collect::<String>()),
                "branch" =>
                    ParseInstr::Branch(
                        get_word(&mut split)?.try_into().ok()?,
                        split.collect::<String>()
                    ),
                "call" =>
                    ParseInstr::Call(split.collect::<String>()),
                "return" =>
                    Instr(Return),
                "left" =>
                    Instr(Left(get_word(&mut split)?.try_into().ok()?)),
                "right" =>
                    Instr(Right(get_word(&mut split)?.try_into().ok()?)),
                "move" =>
                    Instr(Move),
                "pointer" =>
                    Instr(Pointer),
                "add" =>
                    Instr(Add(get_word(&mut split)?.try_into().ok()?)),
                "subtract" =>
                    Instr(Subtract(get_word(&mut split)?.try_into().ok()?)),
                "multiply" =>
                    Instr(Multiply(get_word(&mut split)?.try_into().ok()?)),
                "divide" =>
                    Instr(Divide(get_word(&mut split)?.try_into().ok()?)),
                "compare" =>
                    Instr(Compare(get_word(&mut split)?.try_into().ok()?)),
                "and" =>
                    Instr(And(get_word(&mut split)?.try_into().ok()?)),
                "or" =>
                    Instr(Or(get_word(&mut split)?.try_into().ok()?)),
                "not" =>
                    Instr(Not(get_word(&mut split)?.try_into().ok()?)),
                "cast" =>
                    Instr(Cast(
                        get_word(&mut split)?.try_into().ok()?,
                        get_word(&mut split)?.try_into().ok()?
                    )),
                _ => return None
            };

            instructions.push(parse_instr);
        }

        instructions.iter().map(|val| -> Option<Instruction> { Some(match val {
            ParseInstr::Instr(instr) => instr.clone(),
            ParseInstr::Jump(id) => Jump(*labels.get(id)?),
            ParseInstr::Branch(ty, id) => Branch(
                *ty,
                *labels.get(id)?
            ),
            ParseInstr::Call(id) => Call(*labels.get(id)?)
        })}).collect::<Option<Vec<Instruction>>>()
    }
}