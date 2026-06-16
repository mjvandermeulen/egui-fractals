# TODO

## NOW

- [x] try to paint a random line
- [x] play with centering: x 0 y -2.0
- [x] change `hands` to `design`.
    - [x] `initiator` (`gen_0`). Always from (0,0) to (0,1) _for now_
    - [x] `generator` (`gen_1`)
- [x] HOVER
    - [x] add ~~screen~~local_hover_pos to struct
    - [x] from the hover pos calculate the gen_1 design line
        - [x] using ~~atan2~~ built in vec.angle
    - [x] only repaint if the new hover doesn't match struct hover.
    - [x] always show hover coords

## SUPER NOW

- [x] Alt: only show 1st and 2nd gen
- [ ] on free/tree/loop switch: update design_lines
- [x] double click on line: switch direction

## FIX

- [ ] use "initiator" and "generator"

## NOTE

- [ ] I have change the naming to vec and rot (~~dir vs rot: dir is absolute. rot is relative.~~)
- [ ] vec and rot INCLUDE SCALING!
- [ ] ~~don't allow dragging outside painter.rect() (although you can zoom out...)~~

## LATER

- [x] scrolling:~~ update all desing line coords? Or~~ have a different center to the screen!!!!
- [x] only use UI measurements in `self.design` and state (struct) and return the "screen" positions.
    - [x] limit `to_` and `from_screen` to the `self.design` method.

## MUCH LATER

- [ ] show design params in ui and allow to manually set the numbers.

# THOUGHTS

- the width of the line is independent of the screen size!

# Design

keys:

- up down: level
- 0..8: depth level
- 9: deepest level

- press and hold d (or SHIFT-D to toggle): Show design
- alt 10x fine tune
- ctrl: 100x fine tune

- two finger scroll
- pinch zoom

# Structs and such:

## Node

A Node is like the new view you create on a phone when you stretch, rotate and move an image on a phone with two fingers. It's defined by a position and a vector.

Zoom: ===> Node, Zoom is the top zoom in the settings...

- position
- vector
- line_width

## Transformation

- Translation
- Rotation (includes scaling)

# Thoughts

## width of line

~~The vector length determines the thickness of the line. maybe....~~ Not the case now, but I'd like it.

What if you start the first gen branches with thickness 1?

# FIX HANGING ON START:

```
cd /Users/mjvandermeulen/Library/Application Support/egui-demo-app
ls
app.ron
```

remove the app.ron file.
