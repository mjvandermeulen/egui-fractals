#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
// TODO more than pref: make this always match the player state
pub enum ReverseMode {
    Reverse,
    ReverseInfinite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde:: Deserialize)]
pub enum PlayerState {
    Stopped, // remove this. see Claude question. Then skip serde and always start with Paused (OR stopped = paused with ln_scale = 0.0)
    Paused(ReverseMode),
    Playing(ReverseMode),
    PlayingReverse,         // stops when ln_scale reaches 0
    PlayingReverseInfinite, // keeps zooming out past 0
}

impl PlayerState {
    pub fn reversing_mode(&self) -> ReverseMode {
        match self {
            Self::Paused(reversing_mode) | Self::Playing(reversing_mode) => *reversing_mode,
            Self::PlayingReverseInfinite => ReverseMode::ReverseInfinite,
            Self::PlayingReverse | Self::Stopped => ReverseMode::Reverse,
        }
    }
}
