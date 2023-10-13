use const_guards::guard;

mod structures {
    #[derive(Clone, PartialEq, Eq, Hash)]
    /// An instance of an emulator.
    #[guard(SIZE > 0x20000)]
    pub struct Emulator<const SIZE: u32> {
        val1: u32,
        val2: u32,
        cur: u32,
        ptr: u32,
        stat: u32,
        memory: [u8; SIZE]
    }
}

pub use structures::Emulator;