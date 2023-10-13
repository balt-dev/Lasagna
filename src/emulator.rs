mod structures {

    #[derive(Clone, PartialEq, Eq, Hash)]
    /// An instance of an emulator. You should probably store this on the heap.
    pub struct Emulator<const SIZE: usize> {
        pub val1:   u32,
        pub val2:   u32,
        pub ptr:    u32,
        pub cur:    u32,
        pub stat:   u32,
        pub memory: [u8; SIZE],
        pub debugger: Option<fn(&mut Self) -> ()>
    }

    impl<const SIZE: usize> Default for Emulator<SIZE> {
        fn default() -> Self {
            Self::new(0, 0, 0, 0x20000, 0)
        }
    }

    impl<const SIZE: usize> Emulator<SIZE> {


        /// Creates a new instance of an emulator, with memory zeroed out.
        ///
        /// # Panics
        /// * The memory is too small to hold the stack (smaller than `0x20000`.)
        /// 
        /// ```should_panic
        /// # use lasagna::emulator::Emulator;
        /// let mut emulator = Emulator::<0>::default();
        /// ```
        pub fn new(val1: u32, val2: u32, ptr: u32, cur: u32, stat: u32) -> Self {
            assert!(SIZE > 0x20000, "memory must be large enough to contain stack");
            Self {
                val1,
                val2,
                ptr,
                cur,
                stat,
                memory: [0; SIZE],
                debugger: None
            }
        }
        
        /// Attach a debugging function to this emulator.
        ///
        /// # Examples
        /// ```rust
        /// # use lasagna::emulator::Emulator;
        /// fn debugger<const SIZE: usize>(emulator: &mut Emulator<SIZE>) {
        ///     println!("VAL1: {:02X}", emulator.val1);
        /// }
        /// 
        /// let mut emu = Emulator::<0x100000>::default()
        ///     .with_debugger(debugger);
        /// 
        /// ```
        pub fn with_debugger(mut self, function: fn(&mut Self) -> ()) -> Self {
            self.debugger = Some(function);
            self
        }
    }
}

pub use structures::Emulator;