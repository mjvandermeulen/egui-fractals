# egui Fractals

A simple app to desing line segment fractals.

## how to design fractals:

- select a starter fractal
- drag the line ends of the design lines
- play with depth, etc etc

### important keys:

| Key      | Function                                                                 |
| -------- | ------------------------------------------------------------------------ |
| d(D)     | show design (toggle)                                                     |
| n        | draw a **n**ew line, while holding this key down                         |
| t        | trash a line by double clicking on the line, while holding this key down |
| up/ down | to change depth                                                          |
| 1..7     | to set depth                                                             |
| 8        | half of 9                                                                |
| 9        | set depth to max allowed                                                 |

### important mouse actions:

- drag any line end
- double click on a line, to reverse it. (or delete it while holding down "t")

## run

### local:

    $ cargo run --release

OR

    $ bacon # and then r for run OR bacon run

to see `log::` output in stdout

    $ bacon # and then l for run-with-log

### web WASM

Push the main branch. This will trigger a rebuild on Github and Github will serve the page through github pages:

    https://mjvandermeulen.github.io/egui-fractals/

If changes are lagging:

- Chrome Dev tools: Application -> Clear Site Data
