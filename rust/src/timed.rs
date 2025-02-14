use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Timed {
    pub segments: Vec<Segment>,
    pub word_segments: Vec<WordSegment>,
    pub language: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Segment {
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub words: Vec<WordSegment>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WordSegment {
    pub word: String,
    pub start: Option<f32>,
    pub end: Option<f32>,
    pub score: Option<f32>,
}
