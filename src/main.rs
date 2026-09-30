mod fen;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process::ExitCode;

const USAGE: &str = "usage: fenview [--compact] [FILE...]";

/// Maps a FEN piece letter to its chess glyph. Black pieces use the solid
/// glyphs and white the outlined ones, which is how most fonts draw them.
fn glyph(piece: char) -> char {
    match piece {
        'K' => '\u{2654}',
        'Q' => '\u{2655}',
        'R' => '\u{2656}',
        'B' => '\u{2657}',
        'N' => '\u{2658}',
        'P' => '\u{2659}',
        'k' => '\u{265A}',
        'q' => '\u{265B}',
        'r' => '\u{265C}',
        'b' => '\u{265D}',
        'n' => '\u{265E}',
        'p' => '\u{265F}',
        other => other,
    }
}

fn main() -> ExitCode {
    let mut compact = false;
    let mut paths: Vec<String> = Vec::new();
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--compact" => compact = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            flag if flag.starts_with("--") => {
                eprintln!("fenview: unknown option {flag}");
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            _ => paths.push(arg),
        }
    }
    let mut had_error = false;

    if paths.is_empty() {
        let stdin = io::stdin();
        had_error |= process(stdin.lock(), "stdin", compact);
    } else {
        for path in &paths {
            match File::open(path) {
                Ok(file) => had_error |= process(BufReader::new(file), path, compact),
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
fn process<R: BufRead>(reader: R, source: &str, compact: bool) -> bool {
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
            Ok(position) if compact => {
                print_compact(source, line_number, trimmed, &position)
            }
            Ok(position) => print_position(source, line_number, trimmed, &position),
            Err(e) => {
                eprintln!("fenview: {source}:{line_number}: {e}");
                had_error = true;
            }
        }
    }

    had_error
}

/// Board with unicode glyphs and the remaining fields folded into one line,
/// so a long file of positions stays scannable.
fn print_compact(source: &str, line_number: usize, raw: &str, position: &fen::Position) {
    println!("== {source}:{line_number} ==");
    println!("{raw}");
    println!("  a b c d e f g h");
    for (i, rank) in position.board.iter().enumerate() {
        let cells: Vec<String> = rank
            .iter()
            .map(|square| match square {
                Some(piece) => glyph(*piece).to_string(),
                None => "\u{00B7}".to_string(),
            })
            .collect();
        println!("{} {}", 8 - i, cells.join(" "));
    }
    println!(
        "{} to move, castling {}, en passant {}, halfmove {}, move {}",
        position.active_color,
        position.castling,
        position.en_passant.as_deref().unwrap_or("-"),
        position.halfmove_clock,
        position.fullmove_number
    );
    println!();
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
