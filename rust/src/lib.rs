use anyhow::*;
use glob::glob;
use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod search;

use search::*;

pub fn joined_path() -> Result<Vec<(usize, usize, f32)>> {
    // Compile the regex to capture X, Y, Z from filenames like "path-X-Y-Z.json"
    let re = Regex::new(r"path-(\d+)-(\d+)-(\d+)\.json$")?;

    // A small struct to hold the filename parts + the actual path
    #[derive(Debug)]
    struct FileMeta {
        path: String,
        i: usize,
        dy: usize,
        dx: usize,
    }

    // Collect all files matching the pattern (using a wildcard).
    // Adjust the pattern to your actual directory if needed, e.g. "some_dir/path-*-*-*.json"
    let mut files = Vec::new();
    for entry in glob("./log/path-*-*-*.json")? {
        let path_str = entry?.to_string_lossy().to_string();
        if let Some(filename) = Path::new(&path_str).file_name().and_then(|s| s.to_str()) {
            // Use the regex to parse X, Y, Z
            if let Some(caps) = re.captures(filename) {
                let i = caps[1].parse::<usize>()?;
                let dy = caps[2].parse::<usize>()?;
                let dx = caps[3].parse::<usize>()?;
                files.push(FileMeta {
                    path: path_str,
                    i,
                    dy, // order shit to make consistent with left-right analogy
                    dx,
                });
            }
        }
    }

    // Sort by X ascending
    files.sort_by(|a, b| a.i.cmp(&b.i));

    // This will store the final joined vector
    let mut all_data: Vec<(usize, usize, f32)> = Vec::new();

    // Process each file in sorted order
    for file_meta in files {
        let file = File::open(&file_meta.path)?;
        let reader = BufReader::new(file);

        // The file contains Vec<(usize, usize)>
        let data: Vec<(usize, usize, f32)> = serde_json::from_reader(reader)?;

        // Apply the Y and Z deltas. If the item is (a, b), then the updated item is (a + Y, b + Z).
        let updated_data = data
            .into_iter()
            .map(|(a, b, c)| (a + file_meta.dx, b + file_meta.dy, c)) // ordering shit
            .collect::<Vec<_>>();

        // Push into the final collection
        all_data.extend(updated_data);
    }

    // overlaps caused by half-window movement between each path found
    let all_data = remove_backtracks(&all_data);

    Ok(all_data)
}

// /// Removes branches in a path (represented by `(usize, usize)` coordinates) that
// /// were invalidated by backtracking.
// ///
// /// # Example
// ///
// /// ```
// /// // Suppose we have a path where (2,0) gets revisited, causing everything
// /// // after the *first* (2,0) to be truncated before continuing.
// /// let input = vec![
// ///     (0,0),
// ///     (1,0),
// ///     (2,0),
// ///     (3,0),
// ///     (2,0), // backtrack to the point at index 2
// ///     (2,1),
// ///     (2,2),
// ///     (2,0), // backtrack again to the point at index 2 (in the truncated path)
// ///     (2,3)
// /// ];
// ///
// /// let output = remove_backtracks(&input);
// /// assert_eq!(output, vec![
// ///     (0,0),
// ///     (1,0),
// ///     (2,0),
// ///     (2,3)  // all invalidated branches between the old (2,0) and this new (2,0) got removed
// /// ]);
// /// ```
pub fn remove_backtracks(path: &[(usize, usize, f32)]) -> Vec<(usize, usize, f32)> {
    let mut result = Vec::new();

    for &p in path {
        // Check if `p` already appears in `result`; if so, truncate everything
        // after its *first* occurrence. This effectively discards the old path
        // from that point forward, replacing it with the “new” path.
        if let Some(i) = result.iter().position(|&q| q == p) {
            result.truncate(i);
        }
        // Now push the current point onto our canonical path
        result.push(p);
    }

    result
}

// fixed start
pub fn find_path_by_windows(
    ys: &[&str],
    xs: &[&str],
    mut left_start: usize,
    mut right_start: usize,
) -> Result<Vec<(usize, usize, f32)>> {
    let mut iteration = 0;

    // 50 must be quick, dijkstra is getting very slow above 500
    //let score_batch = 50 as usize;
    let score_batch = 200 as usize; // just to have less files...

    std::fs::create_dir("./log").ok();

    loop {
        println!("iteration: {}...", iteration);
        //let flexible_start = false;

        let path_file_name = format!(
            "./log/path-{}-{}-{}.json",
            iteration, left_start, right_start
        );
        let path = if std::fs::metadata(path_file_name.clone()).is_ok() {
            println!("=> skipped");
            let path: Vec<(usize, usize, f32)> =
                serde_json::from_str(&std::fs::read_to_string(path_file_name).unwrap()).unwrap();
            path
        } else {
            // naming 123...
            let left_xs: Vec<&str> = ys
                .iter()
                .skip(left_start as usize)
                .take(score_batch.into())
                .map(|s| *s)
                .collect();
            let right_xs: Vec<&str> = xs
                .iter()
                .skip(right_start as usize)
                .take(score_batch.into())
                .map(|s| *s)
                .collect();

            let cost_matrix = levenshtein_distance_matrix_parallel(&left_xs, &right_xs);

            let similarity_matrix: Vec<Vec<f32>> = cost_matrix
                .iter()
                .map(|inner| inner.iter().map(|x| 1.0 - x).collect())
                .collect();

            let (path, _cost) = find_path(cost_matrix, false);

            std::fs::write(
                format!(
                    "./log/matrix-{}-{}-{}.json",
                    iteration, left_start, right_start
                ),
                serde_json::to_string(&similarity_matrix).unwrap(),
            )
            .unwrap();

            std::fs::write(path_file_name, serde_json::to_string(&path).unwrap()).unwrap();
            println!("=> found path of {} steps", path.len());
            path
        };

        // reached any border is exit condition
        if let Some(last) = path.last() {
            println!("-----");
            println!("1: {} {} {}", left_start, last.0, ys.len());
            println!("2: {} {} {}", right_start, last.1, xs.len());
            if left_start + last.0 + 1 == ys.len() || right_start + last.1 + 1 == xs.len() {
                break;
            }
        } else {
            panic!("ain't");
        }

        let mid = path.get(path.len() / 2).unwrap();

        left_start += mid.1;
        right_start += mid.0;

        iteration += 1;
    }

    let mut path = joined_path()?;
    let start = path.first().unwrap();
    if start.0 == 0 {
        let y_lim = start.1;
        // 1.0 - max cost
        let mut ys: Vec<(usize, usize, f32)> = (0..y_lim).map(|y| (0, y, 1.0)).collect();
        ys.append(&mut path);
        path = ys;
    } else if start.1 == 0 {
        let x_lim = start.0;
        let mut xs: Vec<(usize, usize, f32)> = (0..x_lim).map(|x| (x, 0, 1.0)).collect();
        xs.append(&mut path);
        path = xs;
    } else {
        panic!();
    }
    std::fs::write(
        "./path-of-alignment.json",
        serde_json::to_string(&path).unwrap(),
    )
    .unwrap();

    Ok(path)
}

pub fn levenshtein_distance_matrix_parallel(ys: &[&str], xs: &[&str]) -> Vec<Vec<f32>> {
    use rayon::prelude::*;
    use triple_accel::levenshtein;

    ys.par_iter()
        .map(|l| {
            xs.par_iter()
                .map(|r| {
                    let distance = levenshtein(l.as_bytes(), r.as_bytes()) as f32;
                    let longest_len = l.len().max(r.len()) as f32;
                    let value = if longest_len > 0.0 {
                        distance / longest_len
                    } else {
                        0.0
                    };
                    value
                })
                .collect()
        })
        .collect()
}
