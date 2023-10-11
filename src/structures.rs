use core::hint::unreachable_unchecked;
use crate::constants;
pub enum Type {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    //Float,
    String
}

pub enum Instruction {
    Noop,
    Load(Type),
    Take(Type),
    Put(Type),
    Discard(Type),
    Copy(Type),
    Random(Type),
    Swap(Type),
    Jump(u32),
    JumpZero(u32),
    Index,
    Return,
    Left(Type),
    Right(Type),
    Start,
    End,
    Add(Type),
    Subtract(Type),
    Multiply(Type),
    Divide(Type),
    Remainder(Type),
    Order(Type),
    ShiftLeft(Type),
    ShiftRight(Type),
    Cast(Type, Type)
}

impl Instruction {
    /// Parse an instruction from a byte slice. Returns None if there's not enough bytes in the slice.
    fn parse(slice: &[u8], index: usize) -> Option<Self> {
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
            0b111 => Type::String,
            // Due to the mask on TYPE, this cannot be reached.
            _ => unsafe {unreachable_unchecked()}
        };

        Some ( match (slice[index] & (!constants::TYPE)) >> 3 {
            0b00_000 => Noop,
            0b00_001 => Load(instr_type),
            0b00_010 => Take(instr_type),
            0b00_011 => Put(instr_type),
            0b00_100 => Discard(instr_type),
            0b00_101 => Copy(instr_type),
            0b00_110 => Random(instr_type),
            0b00_111 => Swap(instr_type),
            0b01_000 => {
                // Check for both overflow and
                if (index > usize::MAX - 4) || (slice.len() < (index + 4)) { return None; }
                let jump = u32::from_be_bytes(
                    // Use an inclusive slice to prevent potential overflow
                    unsafe {slice[index + 1 ..= index + 4].try_into().unwrap_unchecked()}
                );
                Jump(jump)
            },
            0b01_001 => {
                // Code duplication? I think you mean less overhead!
                if (index > usize::MAX - 4) || (slice.len() < (index + 4)) { return None; }
                let jump = u32::from_be_bytes(
                    unsafe {slice[index + 1 ..= index + 4].try_into().unwrap_unchecked()}
                );
                JumpZero(jump)
            },
            0b01_010 => Index,
            0b01_011 => Return,
            0b01_100 => Left(instr_type),
            0b01_101 => Right(instr_type),
            0b01_110 => Start,
            0b01_111 => End,
            0b10_000 => Add(instr_type),
            0b10_001 => Subtract(instr_type),
            0b10_010 => Multiply(instr_type),
            0b10_011 => Divide(instr_type),
            0b10_100 => Remainder(instr_type),
            0b10_101 => Order(instr_type),
            0b10_110 => ShiftLeft(instr_type),
            0b10_111 => ShiftRight(instr_type),
            0b11_000 => Cast(instr_type, Type::U8),
            0b11_001 => Cast(instr_type, Type::I8),
            0b11_010 => Cast(instr_type, Type::U16),
            0b11_011 => Cast(instr_type, Type::I16),
            0b11_100 => Cast(instr_type, Type::U32),
            0b11_101 => Cast(instr_type, Type::I32),
            0b11_110 => Cast(instr_type, Type::Float),
            0b11_111 => Cast(instr_type, Type::String),
            // We matched all 32 possibilities above.
            _ => unsafe {unreachable_unchecked()}
        })
    }
}