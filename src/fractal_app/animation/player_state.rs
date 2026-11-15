use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
// TODO more than pref: make this always match the player state
pub enum ReverseMode {
    Reverse,
    ReverseInfinite,
}
impl fmt::Display for ReverseMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Reverse => "Reverse",
            Self::ReverseInfinite => "Reverse Infinite",
        };
        f.pad(s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde:: Deserialize)]
// See Claude question. Then skip serde and always start with Paused (OR stopped = paused with ln_scale = 0.0)
pub enum PlayerState {
    // NOTE: "Stopped" is Paused with ln_scale == 0.0
    Paused(ReverseMode),
    PlayingForward(ReverseMode),
    PlayingReverse,         // stops when ln_scale reaches 0
    PlayingReverseInfinite, // keeps zooming out past 0
}
