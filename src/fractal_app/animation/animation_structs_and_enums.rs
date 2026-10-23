#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct Animation {
    pub cycle_length: f32,
    // TODO: zoom branch.
}
