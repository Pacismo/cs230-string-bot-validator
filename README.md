# CS230 String Bot Validator

This is the repository for a validator I wrote for the CS230 "string bot" project. This is a modified version of the actual testing code I wrote for my project (which hasn't been published because it relies on arcane Rust and C knowledge).

To use this software, there are two modes:

- A `once` mode, which only tests the client once before exiting. You may optionally pass, as an argument, the path to the executable you are testing.
- An `until-interrupt` mode, which makes a persistent server.

The server runs on the localhost. You may specify other parameters, like message count and maximum message length. You may run `help` for more information.

## Building the Project

You need to install `rustup` to your system. You should consult your package manager or you may install it from the [Rust website](https://www.rust-lang.org/learn/get-started).

Once installed, install the toolchain appropriate for your system. Typically, `rustup toolchain install stable` is sufficient. You should also install the Rust analyzer via `rustup component add rust-analyzer` if you plan on inspecting the source code for yourself.

Run `cargo build` to build the debug version of the code.

Run `cargo build --release` to build the release version of the code.

Run `cargo run --release -- <PARAMS>` to run the validator; pass the parameters you would otherwise pass to the validator in the place of `<PARAMS>`.

## Usage Notes

If you plan on debugging your code, do not specify the executable in `once` mode or use `until-interrupt`.

The port will always change every time you launch the validator. This is because, rather than hard-coding a port for the validator to use, the validator asks the operating system to assign it a port.

## Implementation Notes

I chose to use Tokio instead of OS threads because I wanted to. It also shows the capabilities of Rust as-is and how easy Cargo is to use.

I used CLAP to read command line input for the runtime. It automatically generates documentation and help information. This shows Rust's rich macro system (the macros used are called "derive macros", and there are LISP-like procedural macros (such as `println!`)).

Rust doesn't have any randomization features built-in. For that, the `rand` crate must be imported.

Rust doesn't have support for `async` closures. This is an unstable feature that requres the nightly toolchain.

This code can technically be run on MacOS and Windows (and pretty much any other system with a Rust compiler and a network adapter).

This code only supports localhost sockets *for now*.
