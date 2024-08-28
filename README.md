# NFP RSP stub

## Overview

This project implements a Remote Server Protocol (RSP) stub, enabling firmware
developers to debug RFPC cores on the NFP using a standard GDB client. The RSP
stub operates as a TCP/IP server, hosted on the localhost IP address and using
port 12727. The RSP stub communicates with the RISC-V debuggers on the NFP
over PCIe.

Testing was conducted with a GDB client built for a bare-metal RISC-V 64-bit
architecture. This GDB client recognizes only the General Purpose Registers
(GPRs) of the RISC-V 64-bit architecture, lacking knowledge of the Control and
Status Registers (CSRs). Fortunately, GDB allows for the target description to
be overridden with an alternative register map via XML files. This method
was employed here, and the XML file defining the target description
(`riscv64-arch.xml`) is included in this repository.

## Build and installation instructions

### GDB client

The GDB client can be built using the RISC-V GNU toolchain, which is hosted on
GitHub at
[RISC-V GNU Toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain).
Detailed installation instructions are provided in the repository's Markdown
README file. Once installed, the binary is typically located in
`/opt/riscv/bin`, with the executable named `riscv64-unknown-elf-gdb`. To run
`riscv64-unknown-elf-gdb` from the Command Line Interface (CLI), ensure that
`/opt/riscv/bin` is added to your `PATH` variable. For convenient access, it is
recommended to export the path in your `.bashrc` file and then source it:

```bash
echo export PATH='${PATH}:/opt/riscv/bin' >> "${HOME}/.bashrc"
source "${HOME}/.bashrc"
```

### RSP server stub

The RSP server stub was implemented in Rust. Rust is a systems programming
language renowned for its type and memory safety, as well as its high
performance.

For both CentOS and Ubuntu, the preferred method to install Rust is using
`rustup`.

To download and run the installation script, execute the following command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Select option 1 to proceed with the default installation.
After the installation is complete, configure your shell by sourcing the Rust
environment setup script. This step is typically done automatically, but you can
manually run:

```bash
source $HOME/.cargo/env
```

To verify that Rust has been installed successfully, check the version with:

```bash
rustc --version
```

If the version is displayed correctly, the installation was successful. Cargo,
Rust's build system, is used to compile the binaries. You can compile the
binaries by running:

```bash
cargo build
```

from the top-level directory of this repository. To install the binaries and
make them accessible via the CLI, execute:

```bash
cargo install --path .
```

from the top-level directory of this repository. This will add the binaries to
`$HOME/.cargo/bin` which should be in your PATH variable after sourcing the
environment above.

## Examples

### Getting started

To launch the RSP stub server in the background, execute:

```bash
nfp-rsp -Z 0000:65:00.0 -i rfpc0 -u 0 -g 0 -c 0 > /path/to/log/file 2>&1 &
```

The `-i`, `-u`, `-g`, and `-c` options indicate which core you want to debug
(in this example, the first core of the first group in the first cluster of RFPC
island 0 is being targeted). This command sends both `stdout` and `stderr` to a
log file of your choosing (`/path/to/log/file`). If you don't want to keep a log
of the server output, you can redirect the output to `/dev/null` instead:

```bash
nfp-rsp -Z 0000:65:00.0 -i rfpc0 -u 0 -g 0 -c 0 > /dev/null 2>&1 &
```

After the RSP server is successfully running in the background, you can initiate
the GDB client with the following command:

```bash
riscv64-unknown-elf-gdb /path/to/your/elf/file
```

`/path/to/your/elf/file` refers to the ELF file you want to debug. Ensure that
this ELF file is compiled with the `-g` option so that GDB can access the symbol
table.

Once in the GDB terminal, load the target architecture XML file. This will allow
visibility of all RFPC GPRs and CSRs during the debugging process:

```bash
gdb> set tdesc filename /path/to/riscv64-arch.xml
```

After loading the XML file for the target architecture, connect the GDB client
to the RSP server stub with the following command:

```bash
gdb> target remote 127.0.0.1:12727
```

### Loading firmware

Once connected to the server, the following command will load the firmware from
the ELF file specified when the GDB client was started:

```bash
gdb> load
```

GDB will set the program counter to the start of the application directly after
loading the firmware in the background. The RSP server only supports loading
firmware into CTM or LMEM. EMEM is not yet supported, but support will be added
at a future date.

### Reading and writing rfpc registers

To read all the General Purpose Registers (GPRs) and Control and Status
Registers (CSRs) of the core you can run the following command:

```bash
gdb> info registers
```

GDB reads from these registers when connecting to the server for the first time
and at various other points (like when a breakpoint is hit or when stepping
through the code).

If you want to read a specific register directly from the core you can run
the `print` command:

```bash
gdb> print $mstatus
```

If you want to write to a register you can run the `set` command:

```bash
gdb> set $mhartid
```

### Reading memory

The `x` command can be used to read memory from the NFP. Memory addresses
are from the perspective of the RFPC core, so the memory address has to be
an address in the RFPC address space. EMEM, CTM and LMEM are currently
supported. CLS reads are not yet supported, but these can be included in a later
version of the RSP server stub.

To read memory you can run the following command:

```bash
gdb> x/[count][format][size] <address>
```

where:

* `count`: Number of units to display.
* `format`: Display format (`x` for hexadecimal, `d` for decimal, `s` for
string, `i` for instructions, etc.).
* `size`: Memory unit size (`b` for byte, `h` for halfword, `w` for word, `g`
for giant/quadword).
* `address`: Memory address to read from (must be a valid address from the
RFPC address space).

The following example reads 4 words in hexadecimal format starting at address
`0x1009e00000004` (CTM memory):

```bash
gdb> x/4xw 0x1009e00000004
```

You can also use a variable or symbol name instead of a raw address:

```bash
gdb> x/4x &variable_name
```

### Writing memory

To write to memory you can run the following command:

```bash
gdb> set {<type>} <address> = <value>
```

where:

* `type`: Specifies the type of the value being written
(`int`, `char`, `float`, etc.).

For example, to set a word at address `0x1009e00000004` to the value
`0xdeadbeef`:

```bash
gdb> set {int} 0x1009e00000004 = 0xdeadbeef
```

To modify a variable directly you can use:

```bash
gdb> set variable_name = 42
```

### Stepping through code

Stepping through code in GDB allows you to execute your program line-by-line or
instruction-by-instruction to observe its behavior. Here are the key commands to
step through code:

To execute the next line of code, stepping into functions (if the next line is a
function call, step will enter the function), run:

```bash
gdb> step
```

To execute the next line of code, stepping over functions (if the next line is a
function call, next will execute the entire function and stop at the next line
after the call), run:

```bash
gdb> next
```

This steps one machine instruction at a time, stepping into function calls at
the instruction level:

```bash
gdb> stepi
```

Steps one machine instruction at a time, stepping over function calls at the
instruction level:

```bash
gdb> nexti
```

### Setting breakpoints

There are 3 ways of creating breakpoints with GDB, shown below:

```bash
gdb> break <filename>:<line_number>
gdb> break <function_name>
gdb> break *<address>
```

where:

* `filename`: This is the filename where you want to set the breakpoint. It must
have a `.c`, `.S`, `.s` or `.asm` extension.
* `line_number`: This is the line number in the file where you want to set the
breakpoint.
* `function_name`: when setting the breakpoint at the start of a function, this
is the function name.
* `address`: Instruction address for the breakpoint instruction.

This is an example of setting a breakpoint at a specific line in the source
code:

```bash
gdb> break main.c:42
```

This is an example of setting a breakpoint at a specific function name or
assembler label:

```bash
gdb> break my_function
```

This is an example of setting a breakpoint at a specific address in the
executable code:

```bash
gdb> break *0x1009e00000008
```

After setting the breakpoints above you can simply call:

```bash
gdb> continue
```

The core will then halt at the breakpoints you have set.\

Each breakpoint has a unique number assigned when it's created. You can list all
breakpoints and their numbers using the `info breakpoints` command:

```bash
gdb> info breakpoints
```

You can delete a breakpoint with the following command:

```bash
gdb> delete <breakpoint_number>
```
