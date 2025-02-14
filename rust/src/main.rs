mod chunk;
mod search;
mod timed;

use chunk::*;
use quotation_align::*;
use search::*;
use timed::*;

use once_cell::sync::Lazy;
use regex::Regex;

use std::env;

static CHUNK_TEXT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\s+|[^\s]+)").unwrap());

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <timed_words_json_path> <text_path> <output_path>",
            args[0]
        );
        std::process::exit(1);
    }

    let timed_json_path = &args[1];
    let text_path = &args[2];
    let output_path = &args[3];

    let timed_json = std::fs::read_to_string(timed_json_path).unwrap();
    let timed: Timed = serde_json::from_str(&timed_json).unwrap();

    let text = std::fs::read_to_string(text_path).unwrap();
    let chunks: Vec<&str> = CHUNK_TEXT.find_iter(&text).map(|m| m.as_str()).collect();

    let mut current_line = 1;
    let mut current_paragraph = 1;

    // number of line per chunk
    // (only matters on word items in usage)
    let lines: Vec<usize> = chunks
        .iter()
        .map(|chunk| {
            if chunk.contains('\n') {
                current_line += chunk.matches('\n').count();
            }
            current_line
        })
        .collect();

    // number of paragraph per chunk
    // (only matters on word items in usage)
    let paragraphs: Vec<usize> = chunks
        .iter()
        .map(|chunk| {
            if chunk.contains('\n') && chunk.matches('\n').count() > 1 {
                current_paragraph += 1;
            }
            current_paragraph
        })
        .collect();

    // indexes of items containing alphanumeric
    let words: Vec<usize> = chunks
        .iter()
        .enumerate()
        .filter(|(_, chunk)| chunk.chars().any(|c| c.is_alphanumeric()))
        .map(|(i, _)| i)
        .collect();
    use std::collections::HashMap;
    let chunk_to_word_index: HashMap<usize, usize> = words
        .iter()
        .enumerate()
        .map(|(index, &value)| (value, index))
        .collect();

    let processed: Vec<String> = words
        .iter()
        .map(|&index| process_word(chunks[index]))
        .collect();

    // actually all of them, but whatever
    let timed_words_indexes: Vec<usize> = timed
        .word_segments
        .iter()
        .enumerate()
        .filter(|(_, x)| x.word.chars().any(|c| c.is_alphanumeric()))
        .map(|(i, _)| i)
        .collect();

    //let processed_to_timed_index: HashMap<usize, usize> = timed_words_indexes
    //    .iter()
    //    .enumerate()
    //    .map(|(index, &value)| (value, index))
    //    .collect();

    let timed_processed: Vec<String> = timed_words_indexes
        .iter()
        .map(|&index| process_word(&timed.word_segments[index].word))
        .collect();

    // first finding path starting point
    // it is more relevant to audible books with long intros

    // 4x1 ratio should be good,
    // the idea is that generally diagonal path should fit into one of windows
    // windows will be moving half-step on the long side so it should cover
    // edge cases
    let long = 100; // long window dimension of rectangular windows over matrix
    let short = 25; // this one will

    let left = processed;
    let right = timed_processed;

    let step = long / 2;

    let xs: Vec<&str> = right[0..short].iter().map(String::as_str).collect();

    #[derive(Debug)]
    struct Start {
        point: (usize, usize, f32),
        cost: f32,
    }

    // going along two matrix borders to find the starting of alignment

    use rayon::prelude::*;
    let found1: Option<Start> = (0..((left.len() + step - 1) / step))
        .into_par_iter()
        .map(|i| i * step)
        .filter_map(|y_start| {
            let y_end = (y_start + long).min(left.len());

            if y_start > 0 && y_start + long < y_end {
                return None; // ignoring cut windows at the end
            }

            let ys: Vec<&str> = left[y_start..y_end].iter().map(String::as_str).collect();
            let cost_matrix = levenshtein_distance_matrix_parallel(&ys, &xs);
            let (path, cost) = find_path(cost_matrix, true);
            // TODO: find_path - two params, flexible row and flexible column
            //                   both false - just corner!

            if path.first().unwrap().0 != 0 {
                // starts at the matrx edge, not at the window cut
                return None; // ideally to prevent on search level
            }

            Some(Start {
                point: *path.first().unwrap(),
                cost,
            })
        })
        .reduce_with(|best, current| {
            if current.cost < best.cost {
                current
            } else {
                best
            }
        });

    let ys: Vec<&str> = left[0..short].iter().map(String::as_str).collect();

    let found2: Option<Start> = (0..((right.len() + step - 1) / step))
        .into_par_iter()
        .map(|i| i * step)
        .filter_map(|x_start| {
            let x_end = (x_start + long).min(right.len());

            if x_start > 0 && x_start + long < x_end {
                return None; // ignoring cut windows at the end
            }

            let xs: Vec<&str> = right[x_start..x_end].iter().map(String::as_str).collect();
            let cost_matrix = levenshtein_distance_matrix_parallel(&ys, &xs);
            let (path, cost) = find_path(cost_matrix, true);

            if path.first().unwrap().1 != 0 {
                // starts at the matrx edge, not at the window cut
                return None; // ideally to prevent on search level
            }

            Some(Start {
                point: *path.first().unwrap(),
                cost,
            })
        })
        .reduce_with(|best, current| {
            if current.cost < best.cost {
                current
            } else {
                best
            }
        });

    let way: Start = vec![found1, found2]
        .into_iter()
        .filter_map(|x| x)
        .reduce(|best, current| {
            if current.cost < best.cost {
                current
            } else {
                best
            }
        })
        .unwrap();

    let xs: Vec<&str> = right.iter().map(String::as_str).collect();
    let ys: Vec<&str> = left.iter().map(String::as_str).collect();

    let mut path = find_path_by_windows(&ys, &xs, way.point.1, way.point.0).unwrap();

    println!("{:?}", path.len());
    path.retain(|x| x.2 <= 0.5); // being optimistic, covers "O!" somehow
                                 // can't be more optimistic because then garbage removes things by
                                 // being in the same line on the path...
                                 // maybe should check better/smarter/more-detailed deduplication
                                 //path.retain(|x| x.2 < 0.35); // cost is low, matching is good

    let mut counts: HashMap<usize, usize> = HashMap::new();
    for (_, word_index, _) in &path {
        *counts.entry(*word_index).or_insert(0) += 1;
    }
    println!("{:?}", path.len());
    path.retain(|x| counts[&x.1] == 1); // no ambiguity even after filtering
    println!("{:?}", path.len());

    let mut word_timing: HashMap<usize, WordSegment> = HashMap::new();

    for (timing_path_index, word_index, _) in &path {
        let segment = &timed.word_segments[timed_words_indexes[*timing_path_index]];
        word_timing.insert(*word_index, segment.clone());
    }

    let timed_chunks: Vec<Chunk> = chunks
        .into_iter()
        .enumerate()
        .map(|(i, text)| {
            let word_index = chunk_to_word_index.get(&i);

            let mut st: Option<f32>;
            let mut et: Option<f32>;
            let w: Option<u8>;
            //let tc: Option<f32>;
            //let mut c: f32 = 1.; // max by default, they are meant to be ignored in terms of to audio alignment

            if let Some(&index) = word_index {
                w = Some(1);
                //tc = None;
                st = None;
                et = None;

                let found = word_timing.get(&index);

                if let Some(got) = found {
                    st = got.start;
                    et = got.end;
                }
            } else {
                w = None;
                st = None;
                et = None;
                //tc = None;
            }

            Chunk {
                w,
                s: text.to_string(),
                l: *lines.get(i).unwrap(),
                p: *paragraphs.get(i).unwrap(),
                st,
                et,
                //c,
            }
        })
        .collect();

    std::fs::write(output_path, serde_json::to_string(&timed_chunks).unwrap()).unwrap();
}

fn process_word(input: &str) -> String {
    use deunicode::deunicode;
    let clean: String = input.chars().filter(|c| c.is_alphanumeric()).collect();
    deunicode(&clean).to_lowercase()
}
