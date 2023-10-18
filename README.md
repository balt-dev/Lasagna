### Licensing

This README is in the public domain (as per esolang.org's requirements), while the implementation in this repository is under the MIT license.

---

# Lasagna

Lasagna is an esoteric 32-bit assembly instruction set.

There are four 4-byte registers, being `VAL1`, `VAL2`, `CUR`, and `PTR`.

`VAL1` and `VAL2` hold 4-byte values, `PTR` holds a pointer to a read/write index in memory,
and `CUR` holds a pointer to the next instruction in memory.

`VAL1` and `VAL2` start out with indeterminate values, `PTR` starts at `00000000`, and `CUR` starts at `00020000`.

Memory indexes `00010000` - `0001FFFF` are reserved for the stack.
Writing to these values may or may not be allowed depending on the implementation, don't depend on it.

The first 4 bytes of the stack are the current stack length as a big-endian `u16`,
with two null bytes at the start for padding.
The rest is an array of `values`s.

If the stack is overflowed at any point, an interrupt with code `1` should be raised.

If at any point, an invalid section of memory is accessed, an interrupt of code `1` should be raised.

If `CUR` is at the end of memory after executing an instruction, an interrupt of code `0` should be raised.

Don't depend on the bits in a register not used by an opcode staying the same after said opcode!
They don't necessarily need to stay the same.

Signed integer values are stored using two's complement.

---

## Instructions

An executable is defined by a list of instructions (each being one byte), and corresponding arguments.

Each instruction is formatted as a kind, an index, and, 3 bits of a type (if applicable, else it can be anything).

|          Type           |  Name   | Bits  | Size |
|:-----------------------:|:-------:|:-----:|:----:|
| Unsigned 8-bit integer  |  `u8`   | `000` | `1`  |
|  Signed 8-bit integer   |  `i8`   | `001` | `1`  |
| Unsigned 16-bit integer |  `u16`  | `010` | `2`  |
|  Signed 16-bit integer  |  `i16`  | `011` | `2`  |
| Unsigned 32-bit integer |  `u32`  | `100` | `4`  |
|  Signed 32-bit integer  |  `i32`  | `101` | `4`  |
|          Float          | `float` | `110` | `4`  |
|         Boolean         | `bool`  | `111` | `1`  |

In the following table, N represents the size of the specified type.

|   Instruction    |  Textual Representation  | Description                                                                                                                                                                                                                                                                                                                                                                                                                           |
|:----------------:|:------------------------:|:--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `00` `000` `ANY` |          `noop`          | Does nothing.                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `00` `001` `ANY` |          `push`          | Pushes `VAL1` to the stack.                                                                                                                                                                                                                                                                                                                                                                                                           |
| `00` `010` `ANY` |          `pop`           | Pops `VAL1` from the stack.                                                                                                                                                                                                                                                                                                                                                                                                           |
| `00` `011` `ANY` |       `interrupt`        | Raises an interrupt with the code of `VAL1`.                                                                                                                                                                                                                                                                                                                                                                                          |
| `00` `100` `000` |        `literal`         | Puts a length as a u32, and then bytes of arbitrary data at this instruction, writes it at `PTR`, and skips past it.<br/>Data is formatted as seen in the following section.<br/>This should respect memory mapping.                                                                                                                                                                                                                  |
| `00` `100` `ELS` |          `copy`          | Copies the contents of `VAL1` into `VAL2`.                                                                                                                                                                                                                                                                                                                                                                                            |
| `00` `101` `ANY` |          `swap`          | Swaps the contents of `VAL1` and `VAL2`.                                                                                                                                                                                                                                                                                                                                                                                              |
| `00` `110` `TYP` |      `read [type]`       | Reads N bytes of memory at `PTR` to `VAL1`.                                                                                                                                                                                                                                                                                                                                                                                           |
| `00` `111` `TYP` |      `write [type]`      | Writes N bytes of `VAL1` into memory at `PTR`.                                                                                                                                                                                                                                                                                                                                                                                        |
|       N/A        |       `label [ID]`       | Not represented in the file. Marks a cursor index to jump to.                                                                                                                                                                                                                                                                                                                                                                         |
| `01` `000` `ANY` |       `jump [ID]`        | Jumps to the specified cursor index.                                                                                                                                                                                                                                                                                                                                                                                                  |
| `01` `001` `ANY` |   `branch [type] [ID]`   | Jumps if the value in `VAL1` is zero.                                                                                                                                                                                                                                                                                                                                                                                                 |
| `01` `010` `ANY` | `branchzero [type] [ID]` | Jumps if the value in `VAL1` isn't zero.                                                                                                                                                                                                                                                                                                                                                                                              |
| `01` `011` `ANY` |          `goto`          | Copies `CUR` into `PTR`, moving the pointer to this instruction, and then increments `PTR` by 1 to move past it.                                                                                                                                                                                                                                                                                                                      |
| `01` `100` `TYP` |      `left [type]`       | Decreases `PTR` by N.                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `01` `101` `TYP` |      `right [type]`      | Increases `PTR` by N.                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `01` `110` `ANY` |          `move`          | Copies the contents of `VAL1` into `PTR`.                                                                                                                                                                                                                                                                                                                                                                                             |
| `01` `111` `ANY` |        `pointer`         | Copies `PTR` into the contents of `VAL1`.                                                                                                                                                                                                                                                                                                                                                                                             |
| `10` `000` `TYP` |       `add [type]`       | Adds `VAL1` and `VAL2`, and puts the result in `VAL1`, and any overflow in `VAL2`.                                                                                                                                                                                                                                                                                                                                                    |
| `10` `001` `TYP` |    `subtract [type]`     | Subtracts `VAL1` from `VAL2`, and puts the result in `VAL1`, and any underflow in `VAL2`.                                                                                                                                                                                                                                                                                                                                             |
| `10` `010` `TYP` |    `multiply [type]`     | Multiplies `VAL1` by `VAL2`, and puts the result in `VAL1`, and any overflow in `VAL2`.                                                                                                                                                                                                                                                                                                                                               |
| `10` `011` `TYP` |     `divide [type]`      | Divides `VAL1` by `VAL2`, and puts the quotient in `VAL1`, and the remainder in `VAL2`. Execution halts if `VAL2` is zero.                                                                                                                                                                                                                                                                                                            |
| `10` `100` `TYP` |     `compare [type]`     | Compares `VAL1` with `VAL2`. In `VAL1`, puts, if they're equal, `00`, if `VAL1` is greater, `01`, if `VAL2` is greater, `FF`, and if they're otherwise unequal (e.g. NaN), `7F`.                                                                                                                                                                                                                                                      |
| `10` `101` `TYP` |       `and [type]`       | Performs the bitwise AND of `VAL1` and `VAL2`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                                         |
| `10` `110` `TYP` |       `or [type]`        | Performs the bitwise OR of `VAL1` and `VAL2`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                                          |
| `10` `111` `TYP` |       `not [type]`       | Performs the bitwise NOT of `VAL1`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                                                    |
| `11` `TYP` `TYP` |   `cast [type] [type]`   | Casts the value in `VAL1` from one type to another.<br/>Booleans cast to 0 if false, 1 if true.<br/>Floats cast to 0 for NaN, the minimum value of the type for -Inf, and the maximum value of the type for Inf.</br>Casting a signed number to a boolean tells you if it is smaller than 0, and casting an unsigned number tells you if it is larger than 0.<br/>Integers casting to floats cast to the nearest representable value. |
| `11` `000` `000` |       `shiftleft`        | Performs a bitwise left shift of `VAL1` by the u8 in `VAL2` mod 32, and puts the value in `VAL1`, discarding overflowing bits.                                                                                                                                                                                                                                                                                                        |
| `11` `001` `001` |       `shiftright`       | Performs a bitwise right shift of `VAL1` by the u8 in `VAL2` mod 32, and puts the value in `VAL1`, discarding overflowing bits.                                                                                                                                                                                                                                                                                                       |
| `11` `010` `010` |        `rotleft`         | Performs a bitwise left shift of `VAL1` by the u8 in `VAL2` mod 32, and puts the value in `VAL1`, wrapping overflowing bits to the other side.                                                                                                                                                                                                                                                                                        |
| `11` `011` `011` |        `rotright`        | Performs a bitwise right shift of `VAL1` by the u8 in `VAL2` mod 32, and puts the value in `VAL1`, wrapping overflowing bits to the other side.                                                                                                                                                                                                                                                                                       |
| `11` `100` `100` |     `xor [type N=1]`     | Performs the bitwise OR of the first byte of `VAL1` and `VAL2`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                        |
| `11` `101` `101` |     `xor [type N=2]`     | Performs the bitwise OR of the first 2 bytes of `VAL1` and `VAL2`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                     |
| `11` `110` `110` |     `xor [type N=4]`     | Performs the bitwise OR of the first 4 bytes of `VAL1` and `VAL2`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                     |
| `11` `111` `111` |         `break`          | Raises a breakpoint if the executor is attached to a debugger. Okay for this to be a no-op.                                                                                                                                                                                                                                                                                                                                           |

---

## Textual Representation of Literals

### Comments

Comments are noted as text surrounded by square brackets, with nesting allowed.

`[This is a [comment]]`

### Float

Floats are stored in IEEE 754 format, using 32 bytes to store the value.

All floats MUST be formatted with a number before and after the decimal point, and optionally with an exponent.

Hexadecimal-style floats are not supported.


`0.0`, `-6.2e1`

`00 00 00 00`, `C2 78 00 00`

### Integers

Integers are stored big-endian, two's complement, marked with a suffix.

Any integers that won't fit into their type will fail to compile to a binary program.


`0_u8`, `-1_i8`, `2_u16`, `-3_i16`, `4_u32`, `-5_i32`

`00`, `FF`, `00 02`, `FF FD`, `00 00 00 04`, `FF FF FF FB`

### Boolean

Booleans are either `true` for `01`, or `false` for `00`.

### Strings

Strings are stored as null-terminated ASCII.
They're notated with single quotes around them, escaping with `\`.
`\n` is a newline, and `\t` is a tab.
`\x##` can be used to load arbitrary bytes, but be careful not to use `\x00`, as this terminates the string.

`Hello,\nworld!`

`48 65 6C 6C 6F 2C 0A 77 6F 72 6C 64 21 00`



---

## Examples

```
load literal 2_u32
swap
load literal 3_u32
add
write
```
