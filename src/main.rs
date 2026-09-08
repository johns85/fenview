mod fen;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::ExitCode;

fn main() -> ExitCode {
    let paths: Vec<String> = env::args().skip(1).collect();
    let mut had_error = false;

    if paths.is_empty() {
        let stdin = io::stdin();
        had_error |= process(stdin.lock(), "stdin");
    } else {
        for path in &paths {
            match File::open(path) {
                Ok(file) => had_error |= process(BufReader::new(file), path),
                Err(e) => {
                    eprintln!("fenview: cannot open {path}: {e}");
                    had_error = true;
                }
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Reads FEN records, one per non-blank line, from `reader` and prints a
/// board diagram for each. Lines starting with '#' are treated as comments.
/// Returns true if any line failed to parse.
fn process<R: BufRead>(reader: R, source: &str) -> bool {
    let mut had_error = false;

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("fenview: {source}:{line_number}: read error: {e}");
                had_error = true;
                continue;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        match fen::parse(trimmed) {
            Ok(position) => print_position(source, line_number, trimmed, &position),
            Err(e) => {
                eprintln!("fenview: {source}:{line_number}: {e}");
                had_error = true;
            }
        }
    }

    had_error
}

fn print_position(source: &str, line_number: usize, raw: &str, position: &fen::Position) {
    println!("== {source}:{line_number} ==");
    println!("{raw}");
    println!("  a b c d e f g h");
    for (i, rank) in position.board.iter().enumerate() {
        let rank_label = 8 - i;
        print!("{rank_label} ");
        for square in rank {
            match square {
                Some(piece) => print!("{piece} "),
                None => print!(". "),
            }
        }
        println!();
    }
    println!();
    println!("to move: {}", position.active_color);
    println!("castling: {}", position.castling);
    println!(
        "en passant: {}",
        position.en_passant.as_deref().unwrap_or("-")
    );
    println!("halfmove clock: {}", position.halfmove_clock);
    println!("fullmove number: {}", position.fullmove_number);
    println!();
}
