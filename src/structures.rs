use std::io::{Read, Write, Cursor, Seek};
use std::borrow::Cow;
use rand::{self, Rng};

pub const GROUP: u8 = 0b11000000;
pub const INDEX: u8 = 0b00111000;
pub const TYPE : u8 = 0b00000111;

pub(crate) struct Stack (Cursor<Vec<u8>>);

impl Read for Stack {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for Stack {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let needed_size = (self.0.position() as usize) + buf.len();
        let inner = self.0.get_mut();
        // Noop if already big enough
        if inner.len() < needed_size {
            inner.resize(needed_size, 0);
        }
        let old_pos = self.0.position();
        let amt = self.0.write(buf)?;
        self.0.set_position(old_pos);
        Ok(amt)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

pub struct State<'a, R: Read, W: Write> {
    pub(crate) program: Cursor<&'a [u8]>,
    pub(crate) stack: Stack,
    pub(crate) jump_stack: Vec<u64>,
    input: R,
    output: W
}

macro_rules! operator {
    ($self: ident, $ty: ty, $operator: ident) => {
        let a = $ty::from_be_bytes(a.as_slice().into());
        let b = $ty::from_be_bytes(b.as_slice().into());
        let res = a.$operator(b);
        $self.stack.write_all(&res.0.to_be_bytes())?;
        $self.stack.write_all(&[res.1 as u8])?;
    };
}

macro_rules! order {
    ($self: ident, $ty: ty, $_operator: ident) => {
        use std::cmp::Ordering::*;
        let a = $ty::from_be_bytes(a.as_slice().into());
        let b = $ty::from_be_bytes(b.as_slice().into());
        let order = a.partial_cmp(b);
        let res = match order {
            Some(Equal)   => 0x00,
            Some(Greater) => 0xFF,
            Some(Less)    => 0x01,
            None          => 0x7F
        };
        $self.stack.write_all(&[res as u8])?;
    };
}

macro_rules! checked {
    ($self: ident, $ty: ty, $operator: ident) => {
        let a = $ty::from_be_bytes(a.as_slice().into());
        let b = $ty::from_be_bytes(b.as_slice().into());
        let res = a.$operator(b);
        if let Some(res) = res {
            $self.stack.write_all(&res.to_be_bytes())?;
        }
        $self.stack.write_all(&[res.is_some() as u8])?;
    };
}


macro_rules! numeric {
    ($self: ident $raw_instr: ident $operator: ident $inner: ident) => {
        match $raw_instr | TYPE {
            0b000 => $inner!($self,  u8, $operator),
            0b001 => $inner!($self,  i8, $operator),
            0b010 => $inner!($self, u16, $operator),
            0b011 => $inner!($self, i16, $operator),
            0b100 => $inner!($self, u32, $operator),
            0b101 => $inner!($self, i32, $operator),
            0b110 => $inner!($self, f32, $operator),
            _ => return Err(()) // Invalid instruction
        }
        Ok(())
    }
}

// There's probably a better solution to this but :P
macro_rules! integral {
    ($self: ident $raw_instr: ident $operator: ident $inner: ident) => {
        match $raw_instr | TYPE {
            0b000 => $inner!($self,  u8, $operator),
            0b001 => $inner!($self,  i8, $operator),
            0b010 => $inner!($self, u16, $operator),
            0b011 => $inner!($self, i16, $operator),
            0b100 => $inner!($self, u32, $operator),
            0b101 => $inner!($self, i32, $operator),
            _ => return Err(()) // Invalid instruction
        }
        Ok(())
    }
}

impl<'a, R: Read, W: Write> State<'a, R, W> {
    pub fn new(program: Cursor<&'a [u8]>, reader: R, writer: W) -> Self {
        let stack = Vec::new();
        Self {
            program,
            stack: Stack(Cursor::new(stack)),
            jump_stack: Vec::new(),
            input: reader,
            output: writer
        }
    }

    fn jump_prog(&mut self) -> Result<(), ()> {
        let mut buf = [0; 4];
        self.program.read_exact(&mut buf)
            .map_err(|_| ())?;
        let index = u32::from_be_bytes(
            buf
        ) as usize;
        self.jump_stack.push(self.program.position());
        self.program.set_position(index as u64);
        Ok(())
    }

    /// Read a value
    pub(crate) fn read_value(stream: &mut dyn Read, instr: u8) -> std::io::Result<Vec<u8>> {
        Ok(match instr & TYPE {
            0b000 | 0b001 => {
                let mut buf = [0];
                stream.read_exact(&mut buf)?;
                buf.to_vec()
            },
            0b010 | 0b011 => {
                let mut buf = [0; 2];
                stream.read_exact(&mut buf)?;
                buf.to_vec()
            },
            0b100 | 0b101 | 0b110 => {
                let mut buf = [0; 4];
                stream.read_exact(&mut buf)?;
                buf.to_vec()
            }
            0b111 => {
                let mut buf = Vec::new();
                loop {
                    buf.push(0);
                    let end = buf.len() - 1;
                    stream.read_exact(
                        &mut buf[end..end]
                    )?;
                    if *buf.last().unwrap() == 0 {
                        break;
                    }
                }
                buf
            },
            _ => unreachable!()
        })
    }
    /// Steps the state by one cycle, and returns a boolean of whether or not to halt, or if the stack is overdrawn, an empty Err.
    pub fn step(&mut self) -> Result<bool, ()> {
        let mut instr_buf = [0];
        // Ignore I/O errors, as in the spec
        if let Ok(read_bytes) = self.program.read(&mut instr_buf) {
            if read_bytes == 0 {return Ok(true);}
        }
        let raw_instr = instr_buf[0];
        match (raw_instr & (GROUP | INDEX)) >> 3 {
            0b00000 => {},
            0b00001 => {
                let value = Self::read_value(&mut self.program, raw_instr)
                    .map_err(|_| ())?;
                self.stack.write_all(value.as_slice())
                    .unwrap();
            },
            0b00010 => {
                let value = Self::read_value(&mut self.input, raw_instr);
                if let Ok(v) = &value {
                    self.stack.write_all(v.as_slice())
                        .unwrap();
                };
                self.stack.write_all(&[
                    value.is_ok() as u8
                ]).unwrap();
            },
            0b00011 => {
                let value =
                    Self::read_value(&mut self.stack, raw_instr)
                        .map_err(|_| ())?;
                if let Err(_) = self.output.write_all(value.as_slice()) {
                    // Ignore
                };
            },
            0b00100 => {
                Self::read_value(&mut self.stack, raw_instr)
                    .map_err(|_| ())?;
            },
            0b00101 => {
                let value = Self::read_value(&mut self.stack, raw_instr)
                    .map_err(|_| ())?;
                self.stack.write_all(value.as_slice()).unwrap();
                self.stack.write_all(value.as_slice()).unwrap();
            },
            0b00110 => {
                if raw_instr & TYPE == 0b110 {
                    let bytes = rand::random::<f32>().to_be_bytes();
                    self.stack.write_all(&bytes).unwrap();
                } else {
                    let size = match raw_instr & TYPE {
                        0b000 | 0b001 => 1,
                        0b010 | 0b011 => 2,
                        0b100 | 0b101 => 4,
                        0b111 => return Err(()),
                        _ => unreachable!()
                    };
                    let mut buf = vec![0; size];
                    rand::thread_rng().fill(buf.as_mut_slice());
                    self.stack.write_all(&buf).unwrap();
                }
            }
            0b00111 => {
                let a = Self::read_value(&mut self.stack, raw_instr)
                    .map_err(|_| ())?;
                let b = Self::read_value(&mut self.stack, raw_instr)
                    .map_err(|_| ())?;
                self.stack.write_all(a.as_slice()).unwrap();
                self.stack.write_all(b.as_slice()).unwrap();
            },
            0b01000 => {
                self.stack.0.set_position(0);
            },
            0b01001 => {
                self.jump_prog()?;
            },
            0b01010 => {
                let mut byte = [0];
                self.stack.read_exact(&mut byte)
                    .map_err(|_| ())?;
                let byte = byte[0];
                if byte == 0 {
                    self.jump_prog()?;
                }
            },
            0b01011 => {
                if let Some(idx) = self.jump_stack.pop() {
                    self.program.set_position(idx);
                } else {
                    return Ok(true);
                }
            },
            0b01100 => {
                self.stack.0.set_position(
                    if self.stack.0.position() == 0 {
                        self.stack.0.get_ref().len() as u64
                    } else {
                        self.stack.0.position() - 1
                    }
                );
            },
            0b01101 => {
                self.stack.0.set_position(
                    if self.stack.0.position() ==
                        (self.stack.0.get_ref().len().saturating_sub(1) as u64) {
                        0
                    } else {
                        self.stack.0.position() + 1
                    }
                );
            },
            0b01110 => {
                let idx = self.stack.0.position() as u32;
                let bytes = idx.to_be_bytes();
                self.stack.write_all(&bytes).unwrap();
            },
            0b01111 => {
                self.stack.0.set_position(
                    self.stack.0.get_ref().len().saturating_sub(1) as u64
                );
            },
            0b10000 => numeric!(self raw_instr overflowing_add operator),
            0b10001 => numeric!{self raw_instr overflowing_sub operator},
            0b10010 => numeric!{self raw_instr overflowing_mul operator},
            0b10011 => numeric!{self raw_instr checked_div checked},
            0b10100 => numeric!{self raw_instr checked_rem checked},
            0b10101 => numeric!(self raw_instr partial_cmp order),
            0b10110 => {

            },
            0b10111 => {

            },
            _ => {
                // Casting types
                let from = (raw_instr & INDEX) >> 3;
                let to = raw_instr & TYPE;
            }
        }
        Ok(false)
    }
}