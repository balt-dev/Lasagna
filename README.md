### Licensing

This README is in the public domain (as per Esolang's requirements), while the implementation in this repository is under the MIT license.

---

# Lasagna

Lasagna is an esoteric programming language defined by an unsized LIFO stack (hence the name, stack) of bytes, inspired by assembly languages.

---

## Basics

A program file, with extension `.bin.lsg`, is defined by a list of instructions (each being one byte), and corresponding arguments.

If, at any point, the stack is popped without anything on it, the program will immediately halt with an error.

--- 

## Text

A text representation of a program, with extension `.txt.lsg`, is formatted with one instruction per line.

### Comments

Comments in textual form are any text surrounded by square brackets, with nesting allowed.


`[This is a [comment]]`

### Load

There is a special case for load - you can load many bytes at once, and it's compiled to separate load instructions.

There are 4 different ways to format this data: raw bytes, a string, an integer, or a float.

### Bytes

Raw bytes are formatted with a pound sign, arbitrarily many bytes, and another pound sign.


`# 01 23 45 67 89 AB CD EF #`

`01 23 45 67 89 AB CD EF`

### Float

Floats are stored in IEEE 754 format, using 32 bytes to store the value.

All floats MUST be formatted with a number before and after the decimal point, and optionally with an exponent.

Hexadecimal-style floats are not supported.


`0.0`, `-6.2e1`

`00 00 00 00`, `C2 78 00 00`

### Integer

Integers are stored big-endian, two's complement, with 6 distinct types, marked with a suffix.

Any integers that won't fit into their type will fail to compile to a binary program.


`0_u8`, `-1_i8`, `2_u16`, `-3_i16`, `4_u32`, `-5_i32`

`00`, `FF`, `00 02`, `FF FD`, `00 00 00 04`, `FF FF FF FB`

### String

Strings are stored as ASCII, backwards in the stack, with a null byte at the end.

They are stored backwards for ease of printing.


`'Hello, world!'`

`00 21 64 6C 72 6F 77 20 2C 6F 6C 6C 65 48`

---

## Instructions

Any invalid instruction will make the program fail. This may fail at runtime, but it's best for this to fail at compile time, if compiling.

Each instruction is formatted as a kind, an index, and, 3 bits of a type (if applicable, else `000`).

In textual representation, the type of the operation, if applicable, is put after the instruction.


|          Type           |  Name   | Bits  |
|:-----------------------:|:-------:|:-----:|
| Unsigned 8-bit integer  |  `u8`   | `000` |
|  Signed 8-bit integer   |  `i8`   | `001` |
| Unsigned 16-bit integer |  `u16`  | `010` |
|  Signed 16-bit integer  |  `i16`  | `011` |
| Unsigned 32-bit integer |  `u32`  | `100` |
|  Signed 32-bit integer  |  `i32`  | `101` |
|          Float          | `float` | `110` |
|         String          |  `str`  | `111` |


| Instruction | Typed? | Textual Representation | Description                                                                                                                                                                                                                                                                                                                             |
|:-----------:|:------:|:----------------------:|:----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `00` `000`  |   No   |         `noop`         | Does nothing.                                                                                                                                                                                                                                                                                                                           |
| `00` `001`  |  Yes   |         `load`         | Load the next value in the file to the stack.                                                                                                                                                                                                                                                                                           |
| `00` `010`  |  Yes   |         `take`         | Take a value from stdin and put it on the stack plus a true boolean byte, or if reading fails, just puts a false boolean byte.                                                                                                                                                                                                          |
| `00` `011`  |  Yes   |         `put`          | Pop a value from the stack, and write to stdout. Failing to write still pops from the stack.                                                                                                                                                                                                                                            |
| `00` `100`  |  Yes   |       `discard`        | Pop a value from the stack and discard it.                                                                                                                                                                                                                                                                                              |
| `00` `101`  |  Yes   |         `copy`         | Push the value on the stack to the stack again, copying it.                                                                                                                                                                                                                                                                             |
| `00` `110`  |  Yes   |        `random`        | Push one random value to the stack. Invalid for strings. Anything on the range [0, 1) for floats.                                                                                                                                                                                                                                       |
| `00` `111`  |  Yes   |         `swap`         | Swap the last two values on the stack.                                                                                                                                                                                                                                                                                                  |
|     N/A     |   No   |        `label`         | Defines a label that can be jumped to later. Not represented in the file.<br/>In textual representation, this is an arbitrarily long string, and each string is assigned a pointer index at compile time.                                                                                                                               |
| `01` `000`  |   No   |         `jump`         | Unconditionally jumps to the instruction index in the next four bytes in the file.<br/>Like before, this is compiled from a string from textual representation, and it fails to compile if the label doesn't exist.<br/>If a label's byte ID does not exist, then this instruction is invalid.<br/>This pushes to a special jump stack. |
| `01` `001`  |   No   |       `jumpzero`       | Pops a byte from the stack, and if it is zero, jumps to the instruction in the next four bytes in the file.<br/>This pushes to a special jump stack.                                                                                                                                                                                    |
| `01` `010`  |   No   |        `index`         | Pushes the current popping index of the stack as a u32 to the stack.                                                                                                                                                                                                                                                                    |
| `01` `011`  |   No   |        `return`        | Pops a from the jump stack (mentioned above), and jumps one past that jump instruction.<br/>If there is nothing left on the jump stack, then this immediately ends the program without error.                                                                                                                                           |
| `01` `100`  |  Yes   |       `rotleft`        | Moves the index at where values are popped from the stack to the left by the specified type's size, or 1 for strings.<br/>Any pushing to the stack will overwrite previous values if the pointer is not at the end.<br/>Wraps at the borders.                                                                                           |
| `01` `101`  |  Yes   |       `rotright`       | Moves the index at where values are popped from the stack to the right by the specified type's size, or 1 for strings.<br/>Any pushing to the stack will overwrite previous values if the pointer is not at the end.<br/>Wraps at the borders.                                                                                          |
| `01` `110`  |   No   |        `start`         | Move the pop index to the beginning of the stack.                                                                                                                                                                                                                                                                                       |
| `01` `111`  |   No   |         `end`          | Moves the pop index to the end of the stack.                                                                                                                                                                                                                                                                                            |
| `10` `000`  |  Yes   |         `add`          | Pops two values of the specified type from the stack, adds them, and pushes the resulting value, plus a boolean byte indicating if overflow occurred.<br/>Invalid for strings. If you need concatenation, just push both strings and remove the null terminator.                                                                        |
| `10` `001`  |  Yes   |       `subtract`       | Subtracts two values, and pushes the resulting value, plus a boolean byte indicating if overflow occurred. Invalid for strings.                                                                                                                                                                                                         |
| `10` `010`  |  Yes   |       `multiply`       | Multiplies two values, and pushes the resulting value, plus a boolean byte indicating if overflow occurred. Invalid for strings.                                                                                                                                                                                                        |
| `10` `011`  |  Yes   |        `divide`        | Divides two values, and pushes the resulting value, plus a boolean byte indicating if there was a divide by zero. Invalid for strings.                                                                                                                                                                                                  |
| `10` `100`  |  Yes   |      `remainder`       | Gets the remainder after dividing two values, and pushes the resulting value, plus a boolean byte indicating if there was a divide by zero. Invalid for strings.                                                                                                                                                                        |
| `10` `101`  |  Yes   |        `order`         | Pushes a 00 if the two given values are equal, a FF if the first is larger, a 01 if the second is larger, and a 7F if they're otherwise not equal.                                                                                                                                                                                      |
| `10` `110`  |  Yes   |      `shiftleft`       | Bit-shifts a value leftwards by another value, and pushes the resulting value, filling empty space with zeros and discarding bits outside.<br/>Invalid for strings and floats.                                                                                                                                                          |
| `10` `111`  |  Yes   |      `shiftright`      | Bit-shifts a value rightwards by another value, and pushes the resulting value, filling empty space with zeros and discarding bits outside.<br/>Invalid for strings and floats.                                                                                                                                                         |
|    `11`     | Yes x2 |         `cast`         | Casts a value to another type, and if successful, pushes the result, then 01, to the stack.<br/> If the cast is invalid, pushes 00 to the stack.                                                                                                                                                                                        |

Care must be taken to not confuse types of stack values.