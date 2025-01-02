<p align="center">
    <img src="https://github.com/user-attachments/assets/ebe8a9d0-0207-4822-9c09-d6ea13204b80" width="125" height="125" />
</p>
<h1 align="center">Turing Machine Simulator and Visualizer</h1>

An application to simulate and visualize any one-tape Turing Machine. The application has been written in Rust, and uses a custom Turing library and `ggez` crate for the graphics.

## App preview
https://github.com/user-attachments/assets/e7286ab3-0f72-4348-9295-9880cae90d57

## Building
This project is written in Rust, so you can build it easily using `cargo build --release`.

## How to Use
You can execute a file using:
```
turing <filename> <tape_data> [--dark]
```
Where:
- `<filename>`: Name/path of the custom Turing Machine `.tng` file.
- `<tape_data>`: The tape to execute in the Turing Machine (e.g. `aabb`, without accents)
- `[--dark]`: Optional `--dark` parameter at the end, to turn on the dark mode in the application.

## `.tng` File Format
First, take a look at some examples in the ![examples folder](./examples/), to familiarize yourself with the syntax.

As an example here, we define a basic Turing Machine that will convert `0` to `1` and viceversa. If it encounters any other symbol, it will write the same symbol and move the head to the right (except when `blank_symbol` is encountered, then the machine will switch to the state `s1`, thus halting):
```
config {
    name: "A Turing Machine that flips binary numbers until the end"
    blank_symbol: '_'
    head_start: 1
}

states {
    state s0 is initial {
        0,1,R,s0
        1,0,R,s0
        _,_,S,s1
        default,default,R,s0
    }

    state s1 is final {}
}
```

We can execute it and view the simulation like this (replace `turing` with `cargo run` if debugging):
```
turing flip.tng 11001
```

## License
Licensed under the [MIT License](LICENSE.md).

