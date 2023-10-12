### Licensing

This README is in the public domain (as per Esolang's requirements), while the implementation in this repository is under the MIT license.

---

# Lasagna

Lasagna is an esoteric 32-bit assembly instruction set.

There are three 4-byte registers, being `VAL1`, `VAL2`, and `PTR`.

`VAL1` and `VAL2` hold 4-byte values, `PTR` holds a pointer to a place in memory.

Memory indexes `FF000000` - `FFFFFFFF` are reserved for the subroutine stack, but writing or reading these values is allowed (if you're brave).

The first 2 bytes of the subroutine stack are null, and the next 6 are the current stack length as a `u24`.

---

## Instructions

An executable is defined by a list of instructions (each being one byte), and corresponding arguments.

Each instruction is formatted as a kind, an index, and, 3 bits of a type (if applicable, else it can be anything).

|          Type           |  Name   | Bits  | Size |
|:-----------------------:|:-------:|:-----:|:----:|
| Unsigned 8-bit integer  |  `u8`   | `000` |  1   |
|  Signed 8-bit integer   |  `i8`   | `001` |  1   |
| Unsigned 16-bit integer |  `u16`  | `010` |  2   |
|  Signed 16-bit integer  |  `i16`  | `011` |  2   |
| Unsigned 32-bit integer |  `u32`  | `100` |  4   |
|  Signed 32-bit integer  |  `i32`  | `101` |  4   |
|          Float          | `float` | `110` |  4   |
|         Boolean         | `bool`  | `111` |  1   |

In the following table, N represents the size of the specified type.

|   Instruction    | Textual Representation | Description                                                                                                                                                                                                                                                                                                                                                                                                                           |
|:----------------:|:----------------------:|:--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `00` `000` `ANY` |         `noop`         | Does nothing.                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `00` `001` `TYP` |     `load [type]`      | Loads the next N bytes of the assembly into `VAL1`. In text, the type is deduced from the following value.                                                                                                                                                                                                                                                                                                                            |
| `00` `010` `ARG` |     `system [0-7]`     | Executes a system call with the ID of the u16 in `VAL1`, and the number of arguments being the type of the instruction interpreted as a 3-bit unsigned integer.<br/>The arguments of the system call are stored sequentially in memory at `PTR`.                                                                                                                                                                                      |
| `00` `011` `ANY` |      `interrupt`       | Raises a system interrupt with the exit code stored in `VAL1` as an i32.                                                                                                                                                                                                                                                                                                                                                              |
| `00` `100` `ANY` |         `copy`         | Copies the contents of `VAL1` into `VAL2`.                                                                                                                                                                                                                                                                                                                                                                                            |
| `00` `101` `ANY` |         `swap`         | Swaps the contents of `VAL1` and `VAL2`.                                                                                                                                                                                                                                                                                                                                                                                              |
| `00` `110` `TYP` |     `read [type]`      | Writes N bytes of memory at `PTR` to `VAL1`.                                                                                                                                                                                                                                                                                                                                                                                          |
| `00` `111` `TYP` |     `write [type]`     | Writes N bytes of `VAL1` into memory at `PTR`.                                                                                                                                                                                                                                                                                                                                                                                        |
|       N/A        |      `label [ID]`      | Not represented in the file. Marks a cursor index to jump to.                                                                                                                                                                                                                                                                                                                                                                         |
| `01` `000` `ANY` |      `jump [ID]`       | Jumps to the specified cursor index.                                                                                                                                                                                                                                                                                                                                                                                                  |
| `01` `001` `ANY` |     `branch [ID]`      | Jumps if the value in `VAL1` is `00000000`.                                                                                                                                                                                                                                                                                                                                                                                           |
| `01` `010` `ANY` |      `call [ID]`       | Jumps to the specified cursor index, and pushes the current cursor index to the subroutine stack.                                                                                                                                                                                                                                                                                                                                     |
| `01` `011` `ANY` |        `return`        | Pops a cursor index from the return stack and goes to that cursor index. Execution halts if the stack is empty.                                                                                                                                                                                                                                                                                                                       |
| `01` `100` `TYP` |     `left [type]`      | Decreases `PTR` by N.                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `01` `101` `TYP` |     `right [type]`     | Increases `PTR` by N.                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `01` `110` `ANY` |         `move`         | Copies the contents of `VAL1` into `PTR`.                                                                                                                                                                                                                                                                                                                                                                                             |
| `01` `111` `ANY` |       `pointer`        | Copies `PTR` into the contents of `VAL1`.                                                                                                                                                                                                                                                                                                                                                                                             |
| `10` `000` `TYP` |      `add [type]`      | Adds `VAL1` and `VAL2`, and puts the result in `VAL1`, and any overflow in `VAL2`.                                                                                                                                                                                                                                                                                                                                                    |
| `10` `001` `TYP` |   `subtract [type]`    | Subtracts `VAL1` from `VAL2`, and puts the result in `VAL1`, and any underflow in `VAL2`.                                                                                                                                                                                                                                                                                                                                             |
| `10` `010` `TYP` |   `multiply [type]`    | Multiplies `VAL1` by `VAL2`, and puts the result in `VAL1`, and any overflow in `VAL2`.                                                                                                                                                                                                                                                                                                                                               |
| `10` `011` `TYP` |    `divide [type]`     | Divides `VAL1` by `VAL2`, and puts the quotient in `VAL1`, and the remainder in `VAL2`. Execution halts if `VAL2` is zero.                                                                                                                                                                                                                                                                                                            |
| `10` `100` `TYP` |    `compare [type]`    | Compares `VAL1` with `VAL2`. In `VAL1`, puts, if they're equal, `00`, if `VAL1` is greater, `01`, if `VAL2` is greater, `FF`, and if they're otherwise unequal (e.g. NaN), `7F`.                                                                                                                                                                                                                                                      |
| `10` `101` `TYP` |      `and [type]`      | Performs the bitwise AND of N bytes of `VAL1` and `VAL2`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                              |
| `10` `110` `TYP` |      `or [type]`       | Performs the bitwise OR of N bytes of `VAL1` and `VAL2`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                               |
| `10` `111` `TYP` |      `not [type]`      | Performs the bitwise NOT of N bytes of `VAL1`, and puts the result in `VAL1`.                                                                                                                                                                                                                                                                                                                                                         |
| `11` `TYP` `TYP` |  `cast [type] [type]`  | Casts the value in `VAL1` from one type to another.<br/>Booleans cast to 0 if false, 1 if true.<br/>Floats cast to 0 for NaN, the minimum value of the type for -Inf, and the maximum value of the type for Inf.</br>Casting a signed number to a boolean tells you if it is smaller than 0, and casting an unsigned number tells you if it is larger than 0.<br/>Integers casting to floats cast to the nearest representable value. |

---

## Textual Representation of Types

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

### String

Strings are stored as ASCII, backwards in the stack, with a null byte at the end.

They are stored backwards for ease of use.

Newlines can be represented with `\n`. A character can be escaped with `\`.

`'Hello, world!'`

`00 21 64 6C 72 6F 77 20 2C 6F 6C 6C 65 48`

### Boolean

Booleans are either `true` for `01`, or `false` for `00`.