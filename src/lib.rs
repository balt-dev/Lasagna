#![no_std]

#[cfg(any(target_pointer_width = "16", target_pointer_width = "8"))]
compile_error!("A target pointer width of at least 32 is required for this crate");

pub mod structures;
pub mod constants {
    pub const GROUP: u8 = 0b11000000;
    pub const INDEX: u8 = 0b00111000;
    pub const TYPE : u8 = 0b00000111;
}