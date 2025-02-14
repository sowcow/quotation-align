use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Chunk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<u8>, // flag if it is word vs punctuation/whitespace
    pub s: String, // string
    pub l: usize,  // line number
    pub p: usize,  // paragraph number
    //pub c: f32,    // confidence/score of alignment (< already going filtered)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub st: Option<f32>, // start-timing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et: Option<f32>, // end-timing
}
