# Diode CLI

A collection of tools for working with electronics-as-code.

## Installation

Clone this repo, install the [Rust toolchain](https://www.rust-lang.org/tools/install), and run `cargo run`.

## Commands

### `diode convert`
Run `cargo run -- convert` to convert a KiCad netlist to an Atopile project. If the target directory does not exist, it will call `ato create` for you; if you've got an existing project, it can (with permission) overwrite the files based on the components in the netlist. It will generate:

- A `library/*.ato` file for each library part in the netlist.
- A module for each sheet identified in the netlist.
- A root module (named after the project) to stitch all of the sheet modules together.

Known limitations:
- [ ] The converter is not yet aware of generic components.
- [ ] Some information from the netlist is not captured in the generated project (e.g. resistor values, pin types, etc).
- [ ] The generated Atopile project should compile, but will give warnings about manually-specified designators.
