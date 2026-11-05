## How to keep the scale and rotation pretty solid, when redesigning:

### issue:

Is either the rotation, or the scale always correct compared to the "normal, time dictated values"?

OR

can we decouple and when rewinding, we don't get back to the original rotation? hmmmmmm

### idea:

Remember timestamp, scale (and rotation) in state (Option)

```rust
pub struct AnimationZoom {
    rot
    scale
    timestamp
}

imp AnimationZoom {
    pub fn
}
```

a. If Some rot and scale and timestamp 1. calc delta rot and delta scale since last
