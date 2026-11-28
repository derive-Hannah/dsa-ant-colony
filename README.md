# Ant Simulation
This is a basic simulation of an ant colony retrieving food on a hexagonal grid.
It provides each ant a limited set of knowledge (their own location, details of nearby cells, pheremone strength),
and allows larger behaviour to emerge.

## Installation
Firstly, clone this repository
```console
$ git clone --depth 1 https://github.com/derive-Hannah/dsa-ant-colony
```
Then, navigate to the folder and run it with Rust's build tool, cargo
```console
$ cd dsa-ant-colony
$ cargo run --release
```

## Dependencies
This project requires rust and cargo to be installed.
You can get them at https://rust-lang.org/tools/install/

The project dependencies themselves are limited to macroquad.
Macroquad requires nothing other than rust and cargo to build,
no external dependencies or libraries needed.
