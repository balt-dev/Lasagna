mod structures {

    use crate::constants;

    macro_rules! overflowing {
        ($self: ident, $ty: ident, $int: ident, $float: tt, $b1: expr, $b2: expr) => {
            let lhs = Value::from_bytes(&$self.val1, $ty);
            let rhs = Value::from_bytes(&$self.val2, $ty);
            use Value::*;
            let (val, over) = match (lhs, rhs) {
                (U8(a), U8(b)) => {let (v, o) = a.$int(b); (U8(v), o)},
                (I8(a), I8(b)) => {let (v, o) = a.$int(b); (I8(v), o)},
                (U16(a), U16(b)) => {let (v, o) = a.$int(b); (U16(v), o)},
                (I16(a), I16(b)) => {let (v, o) = a.$int(b); (I16(v), o)},
                (U32(a), U32(b)) => {let (v, o) = a.$int(b); (U32(v), o)},
                (I32(a), I32(b)) => {let (v, o) = a.$int(b); (I32(v), o)},
                (Float(a), Float(b)) => (Float(a $float b), false),
                (Bool(a), Bool(b)) => (Bool($b1(a, b) as u8), $b2(a, b)),
                _ => unreachable!()
            };
            $self.val2[0] = over as u8;
            let (val, size) = val.into_bytes();
            for i in 0..size {
                $self.val1[i] = val[i];
            }
        };
    }

    macro_rules! as_convert {
        ($n: ident, $self: ident; f32 = $size: literal) => {
            let value = ($n as f32).to_le_bytes();
            for i in 0..$size {
                $self.val1[i] = value[i];
            }
        };
        ($n: ident, $self: ident; $to: ty = $size: literal) => {
            let value = ($n as $to).to_be_bytes();
            for i in 0..$size {
                $self.val1[i] = value[i];
            }
        }
    }

    #[derive(Copy, Clone, PartialOrd, PartialEq)]
    enum Value {
        U8(u8),
        I8(i8),
        U16(u16),
        I16(i16),
        U32(u32),
        I32(i32),
        Float(f32),
        Bool(u8)
    }

    impl Value {
        fn from_bytes(value: &[u8], ty: u8) -> Self {
            use Value::*;
            match ty & constants::TYPE {
                0b1000.. => unreachable!(),
                0b000 => U8(value[0]),
                0b001 => I8(value[0] as i8),
                0b010 => U16(u16::from_be_bytes(
                    value[0..2].try_into().unwrap()
                )),
                0b011 => I16(i16::from_be_bytes(
                    value[0..2].try_into().unwrap()
                )),
                0b100 => U32(u32::from_be_bytes(
                    value.try_into().unwrap()
                )),
                0b101 => I32(i32::from_be_bytes(
                    value.try_into().unwrap()
                )),
                0b110 => Float(f32::from_le_bytes(
                    value.try_into().unwrap()
                )),
                0b111 => Bool(value[0])
            }
        }

        fn into_bytes(self) -> ([u8; 4], usize) {
            use Value::*;
            match self {
                U8(v) => ([v, 0, 0, 0], 1),
                I8(v) => ([v as u8, 0, 0, 0], 1),
                U16(v) => {
                    let slice = v.to_be_bytes();
                    ([slice[0], slice[1], 0, 0], 2)
                },
                I16(v) => {
                    let slice = v.to_be_bytes();
                    ([slice[0], slice[1], 0, 0], 2)
                },
                U32(v) => (v.to_be_bytes(), 4),
                I32(v) => (v.to_be_bytes(), 4),
                Float(v) => (v.to_le_bytes(), 4),
                Bool(v) => ([v, 0, 0, 0], 1)
            }
        }
    }

    /// An instance of an emulator. You should probably store this on the heap.
    /// The emulator can be iterated over to get individual step results.
    ///
    /// ```rust
    /// # extern crate alloc;
    /// # use alloc::boxed::Box;
    /// # use lasagna::emulator::Emulator;
    /// let emulator = Box::new(
    ///     Emulator::<0x100000>::default()
    /// );
    ///
    /// for result in emulator {
    ///     if let Some(interrupt) = result {
    ///         eprintln!("interrupted with code {}", interrupt);
    ///         break;
    ///     }
    /// }
    /// ```
    #[derive(Clone)]
    pub struct Emulator<const SIZE: usize> {
        pub val1:   [u8; 4],
        pub val2:   [u8; 4],
        pub ptr:    u32,
        pub cur:    u32,
        pub stat:   u32,
        pub memory: [u8; SIZE],
        pub debugger: Option<fn(&mut Self) -> Option<u32>>,
        pub callback: Option<fn(&mut [u8], u32, &[u8]) -> Option<u32>>
    }

    impl<const SIZE: usize> Default for Emulator<SIZE> {
        fn default() -> Self {
            Self::new([0; 4], [0; 4], 0, 0x20000, 0)
        }
    }

    pub type StepResult = Option<u32>;

    impl<const SIZE: usize> Emulator<SIZE> {
        /// Creates a new instance of an emulator, with memory zeroed out.
        ///
        /// # Panics
        /// * The memory is too small to hold the stack (smaller than `0x20000`.)
        /// * The memory is larger than a u32.
        ///
        /// ```should_panic
        /// # use lasagna::emulator::Emulator;
        /// let mut emulator = Emulator::<0>::default();
        /// ```
        /// ```should_panic
        /// # use lasagna::emulator::Emulator;
        /// let mut emulator = Emulator::<{(u32::MAX as usize) + 1}>::default();
        /// ```
        pub fn new(val1: [u8; 4], val2: [u8; 4], ptr: u32, cur: u32, stat: u32) -> Self {
            assert!(SIZE > 0x20000, "memory must be large enough to contain stack");
            assert!(SIZE <= u32::MAX as usize, "memory size won't fit into a u32");
            Self {
                val1,
                val2,
                ptr,
                cur,
                stat,
                memory: [0; SIZE],
                debugger: None,
                callback: None
            }
        }

        /// Attach a debugging function to this emulator.
        /// Returns a potential interrupt code.
        ///
        /// # Examples
        /// ```rust
        /// # use lasagna::emulator::Emulator;
        /// fn debugger<const SIZE: usize>(emulator: &mut Emulator<SIZE>) -> Option<u32> {
        ///     println!("VAL1: {:02X?}", emulator.val1);
        ///     None
        /// }
        ///
        /// let mut emu = Emulator::<0x100000>::default()
        ///     .with_debugger(debugger);
        ///
        /// ```
        pub fn with_debugger(mut self, function: fn(&mut Self) -> Option<u32>) -> Self {
            self.debugger = Some(function);
            self
        }

        /// Attach a memory write callback to this emulator.
        /// This is useful for simulating memory-mapped IO.
        ///
        /// Note that if a callback is attached, memory isn't written automatically!
        /// You need to write in your callback.
        ///
        /// # Examples
        /// ```rust
        /// # use lasagna::emulator::Emulator;
        /// fn callback(mem_slice: &mut [u8], pointer: u32, value: &[u8]) -> Option<u32> {
        ///     println!("Writing {:02X?} at {}", value, pointer);
        ///     for i in 0..mem_slice.len() {
        ///        mem_slice[i] = value[i];
        ///     }
        ///     None
        /// }
        ///
        /// let mut emu = Emulator::<0x100000>::default()
        ///     .with_callback(callback);
        /// ```
        pub fn with_callback(mut self, function: fn(&mut [u8], u32, &[u8]) -> Option<u32>) -> Self {
            self.callback = Some(function);
            self
        }

        fn get_size(instr: u8) -> usize {
            match instr & constants::TYPE {
                0b000 | 0b001 | 0b111 => 1,
                0b010 | 0b011 => 2,
                0b100 | 0b101 | 0b110 => 4,
                _ => unreachable!()
            }
        }

        fn check_size(&self, size: usize) -> bool {
            self.ptr.checked_add(size as u32).is_none() || (self.ptr as usize + size) >= SIZE
        }

        #[must_use]
        fn get_value(&mut self, instr: u8) -> Option<&mut [u8]> {
            let cur = self.cur as usize;

            let type_size = Self::get_size(instr);
            if self.check_size(type_size) {
                return None;
            }
            Some(&mut self.memory[cur+1 ..= cur+type_size])
        }

        /// Push a 4-byte value to the stack. If the stack is full, returns `false`.
        ///
        /// ```rust
        /// # use lasagna::emulator::Emulator;
        /// # let mut emulator = Box::new(Emulator::<0x100000>::default());
        /// if !emulator.push([0x12, 0x34, 0x56, 0x78]) {
        ///     panic!("Stack overflowed!");
        /// }
        /// assert_eq!(
        ///     &emulator.memory[0x10000 .. 0x10008],
        ///     &[0x00, 0x00, 0x00, 0x01, 0x12, 0x34, 0x56, 0x78]
        /// );
        /// ```
        #[must_use]
        pub fn push(&mut self, value: [u8; 4]) -> bool {
            let stack_length_ref: &mut [u8; 2] =
                (&mut self.memory[0x10002 .. 0x10004]).try_into().unwrap();
            let mut stack_length: u16 = u16::from_be_bytes(*stack_length_ref);
            let start = 0x10004 + (stack_length * 4) as usize;
            let end = start + 4;
            if end >= 0x20000 {
                return false;
            }
            stack_length += 1;
            *stack_length_ref = stack_length.to_be_bytes();
            drop(stack_length_ref);
            let value_ref: &mut [u8; 4] =
                (&mut self.memory[start .. end]).try_into().unwrap();
            *value_ref = value;
            true
        }

        /// Pop a 4-byte value from the stack. If the stack is full, returns `None`.
        /// The value is not removed from memory!
        ///
        /// ```rust
        /// // Continuing from `[Emulator::push]`...
        /// # use lasagna::emulator::Emulator;
        /// # let mut emulator = Box::new(Emulator::<0x100000>::default());
        /// # let _ = emulator.push([0x12, 0x34, 0x56, 0x78]);
        /// let value = emulator.pop().unwrap();
        /// assert_eq!(value, [0x12, 0x34, 0x56, 0x78]);
        /// assert_eq!(
        ///     &emulator.memory[0x10000 .. 0x10008],
        ///     &[0x00, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78]
        /// );
        /// ```
        #[must_use]
        pub fn pop(&mut self) -> Option<[u8; 4]> {
            let stack_length_ref: &mut [u8; 2] =
                (&mut self.memory[0x10002 .. 0x10004]).try_into().unwrap();
            let mut stack_length: u16 = u16::from_be_bytes(*stack_length_ref);
            if stack_length == 0 {return None;}
            stack_length -= 1;
            let start = 0x10004 + (stack_length * 4) as usize;
            let end = start + 4;
            *stack_length_ref = stack_length.to_be_bytes();
            drop(stack_length_ref);
            Some(
                self.memory[start .. end].try_into().unwrap()
            )
        }

        /// Step the emulator by 1 opcode, returning if there was an interrupt.
        ///
        /// # Implicit Interrupts
        /// * The cursor is at the very end of the memory after execution. `Some(0)`
        /// * The emulator tried to access out of bounds memory. `Some(1)`
        /// * The stack was overflowed. `Some(2)`
        /// * The stack was popped with nothing on it. `Some(3)`
        /// * There was a divide by 0. `Some(4)`
        #[must_use]
        pub fn step(&mut self) -> StepResult {
            let cur = self.cur as usize;
            if !(0..SIZE).contains(&cur) {
                return Some(1);
            }
            let instr = self.memory[cur];
            match (
                (instr & constants::GROUP) >> 6,
                (instr & constants::INDEX) >> 3,
                instr & constants::TYPE
            ) {
                (0b100.., _, _) => unreachable!(),
                (_, 0b1000.., _) => unreachable!(),
                (_, _, 0b1000..) => unreachable!(),

                (0b00, 0b000, _) => {},
                (0b00, 0b001, _) => if !self.push(self.val1) {
                    return Some(2)
                },
                (0b00, 0b010, _) => {
                    match self.pop() {
                        Some(v) => self.val1 = v,
                        None => return Some(3)
                    }
                },
                (0b00, 0b011, _) => return Some(
                    u32::from_be_bytes(self.val1)
                ),
                (0b00, 0b100, 0b000) => {
                    if self.cur.checked_add(5).is_none() {
                        return Some(1);
                    }
                    self.cur += 1;
                    let length: &[u8; 4] = (
                        &self.memory[self.cur as usize ..= self.cur as usize + 3]
                    ).try_into().unwrap();
                    self.cur += 4;
                    let length = u32::from_be_bytes(*length);
                    if self.cur.checked_add(length).is_none() {
                        return Some(1);
                    }
                    if let Some(callback) = self.callback {
                        let literal =
                            self.memory[self.cur as usize .. (self.cur + length) as usize]
                                .to_owned();
                        callback(
                            &mut self.memory[self.ptr as usize .. (self.ptr + length) as usize],
                            length,
                            &literal
                        )?;
                    } else {
                        for i in 0..length {
                            self.memory[(self.ptr + i) as usize] = self.memory[(self.cur + i) as usize];
                        }
                    }
                    self.cur += length;
                },
                (0b00, 0b100, _) => self.val2 = self.val1,
                (0b00, 0b101, _) => core::mem::swap(&mut self.val1, &mut self.val2),
                (0b00, 0b110, ty) => {
                    let size = Self::get_size(ty) - 1;
                    if self.check_size(size) {return Some(1)};
                    let mem = &self.memory[
                        self.ptr as usize ..= self.ptr as usize + size
                        ];
                    // Probably a way better way to do this but :P
                    for i in 0 ..= size {
                        self.val1[i] = mem[i];
                    }
                },
                (0b00, 0b111, ty) => {
                    let size = Self::get_size(ty) - 1;
                    if self.check_size(size) {return Some(1)};
                    let mem = &mut self.memory[
                        self.ptr as usize ..= self.ptr as usize + size
                        ];
                    if let Some(callback) = self.callback {
                        callback(mem, self.ptr, &self.val1)?;
                    } else {
                        // Probably a way better way to do this but :P
                        for i in 0 ..= size {
                            mem[i] = self.val1[i];
                        }
                    }
                },
                (0b01, 0b000, _) => {
                    if self.check_size(3) { return Some(1); }
                    self.cur = u32::from_be_bytes(
                        self.memory[self.ptr as usize ..= (self.ptr + 3) as usize]
                            .try_into().unwrap()
                    );
                },
                (0b01, 0b001, ty) => {
                    if self.check_size(3) { return Some(1); }
                    let jump = match self.get_value(ty) {
                        Some(v) => v,
                        None => return Some(1)
                    };
                    if jump.iter().all(|v| *v == 0) {
                        self.cur = u32::from_be_bytes(
                            self.memory[self.ptr as usize ..= (self.ptr + 3) as usize]
                                .try_into().unwrap()
                        );
                    }
                },
                (0b01, 0b010, ty) => {
                    if self.check_size(3) { return Some(1); }
                    let jump = match self.get_value(ty) {
                        Some(v) => v,
                        None => return Some(1)
                    };
                    if jump.iter().any(|v| *v != 0) {
                        self.cur = u32::from_be_bytes(
                            self.memory[self.ptr as usize ..= (self.ptr + 3) as usize]
                                .try_into().unwrap()
                        );
                    }
                },
                (0b01, 0b011, _) => {
                    self.cur = match self.ptr.checked_add(1) {
                        Some(v) => v,
                        None => return Some(1)
                    };
                },
                (0b01, 0b100, ty) => {
                    let size = Self::get_size(ty);
                    self.ptr = match self.ptr.checked_add(size as u32) {
                        Some(v) => v,
                        None => return Some(1)
                    };
                },
                (0b01, 0b101, ty) => {
                    let size = Self::get_size(ty);
                    self.ptr = match self.ptr.checked_sub(size as u32) {
                        Some(v) => v,
                        None => return Some(1)
                    };
                },
                (0b01, 0b110, _) => self.ptr = u32::from_be_bytes(self.val1),
                (0b01, 0b111, _) => self.val1 = self.ptr.to_be_bytes(),
                (0b10, 0b000, ty) => {overflowing!(
                    self, ty, overflowing_add, +,
                    |a: u8, b: u8| (a ^ b),
                    |a: u8, b: u8| (a & b) != 0
                );},
                (0b10, 0b001, ty) => {overflowing!(
                    self, ty, overflowing_sub, -,
                    |a: u8, b: u8| (!a & b),
                    |a: u8, b: u8| (!b & a) != 0
                );},
                (0b10, 0b010, ty) => {
                    let lhs = Value::from_bytes(&self.val1, ty);
                    let rhs = Value::from_bytes(&self.val2, ty);
                    use Value::*;
                    let (val, over) = match (lhs, rhs) {
                        (U8(a), U8(b)) => {let v = (a as u16) * (b as u16); (U8((v & 0xFF) as u8), U8((v >> 8) as u8))},
                        (I8(a), I8(b)) => {let v = (a as i16) * (b as i16); (I8(((v as u16) & 0xFF) as i8), I8((v >> 8) as i8))},
                        (U16(a), U16(b)) => {let v = (a as u32) * (b as u32); (U16((v & 0xFFFF) as u16), U16((v >> 16) as u16))},
                        (I16(a), I16(b)) => {let v = (a as i32) * (b as i32); (I16(((v as u32) & 0xFFFF) as i16), I16((v >> 16) as i16))},
                        (U32(a), U32(b)) => {let v = (a as u64) * (b as u64); (U32((v & 0xFFFFFFFF) as u32), U32((v >> 32) as u32))},
                        (I32(a), I32(b)) => {let v = (a as i64) * (b as i64); (I32(((v as u64) & 0xFFFFFFFF) as i32), I32((v >> 32) as i32))},
                        (Float(a), Float(b)) => (Float(a * b), Float(0.0)),
                        (Bool(a), Bool(b)) => (Bool(a & b), Bool(0)),
                        _ => unreachable!()
                    };
                    let (val, size) = val.into_bytes();
                    for i in 0..size {
                        self.val1[i] = val[i];
                    }
                    let (over, size) = over.into_bytes();
                    for i in 0..size {
                        self.val2[i] = over[i];
                    }
                },
                (0b10, 0b011, ty) => {
                    let lhs = Value::from_bytes(&self.val1, ty);
                    let rhs = Value::from_bytes(&self.val2, ty);
                    use Value::*;
                    let (val, over) = match (lhs, rhs) {
                        (U8(a), U8(b)) => {if b == 0 {return Some(4)}; (U8(a / b), U8(a % b))},
                        (I8(a), I8(b)) => {if b == 0 {return Some(4)}; (I8(a.wrapping_div(b)), I8(a.wrapping_rem(b)))},
                        (U16(a), U16(b)) => {if b == 0 {return Some(4)}; (U16(a / b), U16(a % b))},
                        (I16(a), I16(b)) => {if b == 0 {return Some(4)}; (I16(a.wrapping_div(b)), I16(a.wrapping_rem(b)))},
                        (U32(a), U32(b)) => {if b == 0 {return Some(4)}; (U32(a / b), U32(a % b))},
                        (I32(a), I32(b)) => {if b == 0 {return Some(4)}; (I32(a.wrapping_div(b)), I32(a.wrapping_rem(b)))},
                        (Float(a), Float(b)) => {if b == 0.0 {return Some(4)}; (Float((a / b).trunc() * b), Float(a % b))},
                        (Bool(_), Bool(b)) => {if b == 0 {return Some(4)}; (lhs, Bool(0))},
                        _ => unreachable!()
                    };
                    let (val, size) = val.into_bytes();
                    for i in 0..size {
                        self.val1[i] = val[i];
                    }
                    let (over, size) = over.into_bytes();
                    for i in 0..size {
                        self.val2[i] = over[i];
                    }
                },
                (0b10, 0b100, ty) => {
                    let lhs = Value::from_bytes(&self.val1, ty);
                    let rhs = Value::from_bytes(&self.val2, ty);
                    self.val1[0] = match lhs.partial_cmp(&rhs) {
                        Some(ord) => (ord as i8) as u8,
                        None => 0x7F
                    };
                },
                (0b10, 0b101, ty) => {
                    let size = Self::get_size(ty);
                    for i in 0..size {
                        self.val1[i] &= self.val2[i];
                    }
                },
                (0b10, 0b110, ty) => {
                    let size = Self::get_size(ty);
                    for i in 0..size {
                        self.val1[i] |= self.val2[i];
                    }
                },
                (0b10, 0b111, ty) => {
                    let size = Self::get_size(ty);
                    for i in 0..size {
                        self.val1[i] = !self.val1[i];
                    }
                },
                (0b11, 0b000, 0b000) => {
                    let amount = self.val2[0] % 32;
                    let val = u32::from_be_bytes(self.val1) << amount;
                    self.val1 = val.to_be_bytes();
                },
                (0b11, 0b001, 0b001) => {
                    let amount = self.val2[0] % 32;
                    let val = u32::from_be_bytes(self.val1) >> amount;
                    self.val1 = val.to_be_bytes();
                },
                (0b11, 0b010, 0b010) => {
                    let amount = self.val2[0];
                    let val = u32::from_be_bytes(self.val1).rotate_left(amount as u32);
                    self.val1 = val.to_be_bytes();
                },
                (0b11, 0b011, 0b011) => {
                    let amount = self.val2[0] % 32;
                    let val = u32::from_be_bytes(self.val1).rotate_right(amount as u32);
                    self.val1 = val.to_be_bytes();
                },
                (0b11, 0b100, 0b100) => self.val1[0] = self.val1[0] ^ self.val2[0],
                (0b11, 0b101, 0b101) => {
                    self.val1[0] = self.val1[0] ^ self.val2[0];
                    self.val1[1] = self.val1[1] ^ self.val2[1];
                },
                (0b11, 0b110, 0b110) => {
                    // eh screw it
                    self.val1[0] = self.val1[0] ^ self.val2[0];
                    self.val1[1] = self.val1[1] ^ self.val2[1];
                    self.val1[2] = self.val1[2] ^ self.val2[2];
                    self.val1[3] = self.val1[3] ^ self.val2[3];
                },
                (0b11, 0b111, 0b111) => if let Some(debugger) = self.debugger {
                    debugger(self)?;
                },
                (0b11, from, to) => {
                    let old = Value::from_bytes(&self.val1, from);
                    const FLOAT_ONE: [u8; 4] = [0x3F, 0x80, 0x00, 0x00];
                    match (old, to) {
                        // Unreachables
                        (_, 0b1000..)              => unreachable!(),
                        (Value::U8(_),      0b000) => unreachable!(),
                        (Value::I8(_),      0b001) => unreachable!(),
                        (Value::U16(_),     0b010) => unreachable!(),
                        (Value::I16(_),     0b011) => unreachable!(),
                        (Value::U32(_),     0b100) => unreachable!(),
                        (Value::I32(_),     0b101) => unreachable!(),
                        (Value::Float(_),   0b110) => unreachable!(),
                        (Value::Bool(_),    0b111) => unreachable!(),
                        // U8 ->
                        (Value::U8(_), 0b001) => {},
                        (Value::U8(n), 0b010 | 0b011) => {as_convert!(n, self; u16 = 2);}
                        (Value::U8(n), 0b100 | 0b101) => {as_convert!(n, self; u32 = 4);}
                        (Value::U8(n), 0b110) => {as_convert!(n, self; f32 = 4);}
                        (Value::U8(n), 0b111) => {self.val1[0] = (n > 0) as u8},
                        // I8 ->
                        (Value::I8(_), 0b000) => {},
                        (Value::I8(n), 0b010 | 0b011) => {as_convert!(n, self; i16 = 2);}
                        (Value::I8(n), 0b100 | 0b101) => {as_convert!(n, self; i32 = 4);}
                        (Value::I8(n), 0b110) => {as_convert!(n, self; f32 = 4);}
                        (Value::I8(n), 0b111) => {self.val1[0] = (n < 0) as u8},
                        // U16 ->
                        (Value::U16(n), 0b000 | 0b001) => {as_convert!(n, self; u8 = 1);},
                        (Value::U16(_), 0b011) => {}
                        (Value::U16(n), 0b100 | 0b101) => {as_convert!(n, self; u32 = 4);}
                        (Value::U16(n), 0b110) => {as_convert!(n, self; f32 = 4);}
                        (Value::U16(n), 0b111) => {self.val1[0] = (n > 0) as u8},
                        // I16 ->
                        (Value::I16(n), 0b000 | 0b001) => {as_convert!(n, self; i8 = 1);},
                        (Value::I16(_), 0b010) => {}
                        (Value::I16(n), 0b100 | 0b101) => {as_convert!(n, self; i32 = 4);}
                        (Value::I16(n), 0b110) => {as_convert!(n, self; f32 = 4);}
                        (Value::I16(n), 0b111) => {self.val1[0] = (n < 0) as u8},
                        // U32 ->
                        (Value::U32(n), 0b000 | 0b001) => {as_convert!(n, self; u8 = 1);},
                        (Value::U32(n), 0b010 | 0b011) => {as_convert!(n, self; u16 = 2);}
                        (Value::U32(_), 0b101) => {}
                        (Value::U32(n), 0b110) => {as_convert!(n, self; f32 = 4);}
                        (Value::U32(n), 0b111) => {self.val1[0] = (n > 0) as u8},
                        // I32 ->
                        (Value::I32(n), 0b000 | 0b001) => {as_convert!(n, self; i8 = 1);},
                        (Value::I32(n), 0b010 | 0b011) => {as_convert!(n, self; i16 = 2);}
                        (Value::I32(_), 0b100) => {}
                        (Value::I32(n), 0b110) => {as_convert!(n, self; f32 = 4);}
                        (Value::I32(n), 0b111) => {self.val1[0] = (n < 0) as u8},
                        // Float ->
                        (Value::Float(n), 0b000) => {as_convert!(n, self; u8 = 1);},
                        (Value::Float(n), 0b001) => {as_convert!(n, self; i8 = 1);},
                        (Value::Float(n), 0b010) => {as_convert!(n, self; u16 = 2);}
                        (Value::Float(n), 0b011) => {as_convert!(n, self; i16 = 2);}
                        (Value::Float(n), 0b100) => {as_convert!(n, self; u32 = 4);}
                        (Value::Float(n), 0b101) => {as_convert!(n, self; i32 = 4);}
                        (Value::Float(n), 0b111) => {self.val1[0] = (n > 0.0) as u8},
                        // Bool ->
                        (Value::Bool(_), 0b000 | 0b001) => {},
                        (Value::Bool(n), 0b010 | 0b011) => {as_convert!(n, self; u16 = 2);},
                        (Value::Bool(n), 0b100 | 0b101) => {as_convert!(n, self; u32 = 4);},
                        (Value::Bool(n), 0b110) => {self.val1 = if n == 0 {[0; 4]} else {FLOAT_ONE}}
                    }
                }
            }
            if SIZE - 1 == cur {
                Some(0)
            } else {
                self.cur += 1;
                None
            }
        }
    }

    impl<const SIZE: usize> Iterator for Emulator<SIZE> {
        type Item = StepResult;

        fn next(&mut self) -> Option<StepResult> {
            Some(self.step())
        }
    }
}

pub use structures::Emulator;