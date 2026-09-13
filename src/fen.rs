use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut any = false;
        if self.white_kingside {
            write!(f, "K")?;
            any = true;
        }
        if self.white_queenside {
            write!(f, "Q")?;
            any = true;
        }
        if self.black_kingside {
            write!(f, "k")?;
            any = true;
        }
        if self.black_queenside {
            write!(f, "q")?;
            any = true;
        }
        if !any {
            write!(f, "-")?;
        }
        Ok(())
    }
}

/// A parsed, validated FEN record.
///
/// `board[0]` is rank 8 (the row printed at the top of a diagram) and
/// `board[7]` is rank 1, matching how FEN lists ranks top to bottom.
pub struct Position {
    pub board: [[Option<char>; 8]; 8],
    pub active_color: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<String>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

#[derive(Debug)]
pub enum FenError {
    WrongFieldCount(usize),
    WrongRankCount(usize),
    InvalidRankLength { rank: usize, found: usize },
    InvalidPieceChar(char),
    InvalidActiveColor(String),
    InvalidCastling(String),
    InvalidEnPassant(String),
    InvalidHalfmoveClock(String),
    InvalidFullmoveNumber(String),
    InvalidKingCount { color: Color, count: usize },
    PawnOnBackRank(usize),
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenError::WrongFieldCount(n) => {
                write!(f, "expected 6 space-separated fields, found {n}")
            }
            FenError::WrongRankCount(n) => {
                write!(f, "piece placement must have 8 ranks, found {n}")
            }
            FenError::InvalidRankLength { rank, found } => write!(
                f,
                "rank {} does not sum to 8 files (found {})",
                8 - rank,
                found
            ),
            FenError::InvalidPieceChar(c) => write!(f, "'{c}' is not a valid piece letter"),
            FenError::InvalidActiveColor(s) => {
                write!(f, "active color must be 'w' or 'b', found '{s}'")
            }
            FenError::InvalidCastling(s) => write!(f, "invalid castling field '{s}'"),
            FenError::InvalidEnPassant(s) => write!(f, "invalid en passant square '{s}'"),
            FenError::InvalidHalfmoveClock(s) => write!(f, "invalid halfmove clock '{s}'"),
            FenError::InvalidFullmoveNumber(s) => write!(f, "invalid fullmove number '{s}'"),
            FenError::InvalidKingCount { color, count } => {
                write!(f, "expected exactly one {color} king, found {count}")
            }
            FenError::PawnOnBackRank(rank) => {
                write!(f, "pawn cannot stand on back rank {rank}")
            }
        }
    }
}

const PIECE_LETTERS: &str = "KQRBNPkqrbnp";

pub fn parse(fen: &str) -> Result<Position, FenError> {
    let fields: Vec<&str> = fen.split_whitespace().collect();
    if fields.len() != 6 {
        return Err(FenError::WrongFieldCount(fields.len()));
    }

    let board = parse_placement(fields[0])?;
    validate_placement(&board)?;
    let active_color = parse_active_color(fields[1])?;
    let castling = parse_castling(fields[2])?;
    let en_passant = parse_en_passant(fields[3])?;

    let halfmove_clock = fields[4]
        .parse::<u32>()
        .map_err(|_| FenError::InvalidHalfmoveClock(fields[4].to_string()))?;

    let fullmove_number = fields[5]
        .parse::<u32>()
        .map_err(|_| FenError::InvalidFullmoveNumber(fields[5].to_string()))?;
    if fullmove_number == 0 {
        return Err(FenError::InvalidFullmoveNumber(fields[5].to_string()));
    }

    Ok(Position {
        board,
        active_color,
        castling,
        en_passant,
        halfmove_clock,
        fullmove_number,
    })
}

fn parse_placement(field: &str) -> Result<[[Option<char>; 8]; 8], FenError> {
    let ranks: Vec<&str> = field.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError::WrongRankCount(ranks.len()));
    }

    let mut board = [[None; 8]; 8];
    for (rank_index, rank_str) in ranks.iter().enumerate() {
        let mut file_count = 0usize;
        for c in rank_str.chars() {
            if let Some(digit) = c.to_digit(10) {
                if digit == 0 || file_count + digit as usize > 8 {
                    return Err(FenError::InvalidRankLength {
                        rank: rank_index,
                        found: file_count + digit as usize,
                    });
                }
                file_count += digit as usize;
            } else if PIECE_LETTERS.contains(c) {
                if file_count >= 8 {
                    return Err(FenError::InvalidRankLength {
                        rank: rank_index,
                        found: file_count + 1,
                    });
                }
                board[rank_index][file_count] = Some(c);
                file_count += 1;
            } else {
                return Err(FenError::InvalidPieceChar(c));
            }
        }
        if file_count != 8 {
            return Err(FenError::InvalidRankLength {
                rank: rank_index,
                found: file_count,
            });
        }
    }
    Ok(board)
}

/// Checks sanity constraints that a syntactically valid placement string can
/// still violate: each side needs exactly one king, and pawns can't sit on
/// the back rank (they promote before ever reaching it).
fn validate_placement(board: &[[Option<char>; 8]; 8]) -> Result<(), FenError> {
    let mut white_kings = 0usize;
    let mut black_kings = 0usize;

    for (rank_index, rank) in board.iter().enumerate() {
        for square in rank {
            match square {
                Some('K') => white_kings += 1,
                Some('k') => black_kings += 1,
                Some('P') | Some('p') if rank_index == 0 || rank_index == 7 => {
                    return Err(FenError::PawnOnBackRank(8 - rank_index));
                }
                _ => {}
            }
        }
    }

    if white_kings != 1 {
        return Err(FenError::InvalidKingCount {
            color: Color::White,
            count: white_kings,
        });
    }
    if black_kings != 1 {
        return Err(FenError::InvalidKingCount {
            color: Color::Black,
            count: black_kings,
        });
    }
    Ok(())
}

fn parse_active_color(field: &str) -> Result<Color, FenError> {
    match field {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        other => Err(FenError::InvalidActiveColor(other.to_string())),
    }
}

fn parse_castling(field: &str) -> Result<CastlingRights, FenError> {
    if field == "-" {
        return Ok(CastlingRights::default());
    }

    let mut rights = CastlingRights::default();
    for c in field.chars() {
        match c {
            'K' if !rights.white_kingside => rights.white_kingside = true,
            'Q' if !rights.white_queenside => rights.white_queenside = true,
            'k' if !rights.black_kingside => rights.black_kingside = true,
            'q' if !rights.black_queenside => rights.black_queenside = true,
            _ => return Err(FenError::InvalidCastling(field.to_string())),
        }
    }
    Ok(rights)
}

fn parse_en_passant(field: &str) -> Result<Option<String>, FenError> {
    if field == "-" {
        return Ok(None);
    }
    let mut chars = field.chars();
    let file = chars.next();
    let rank = chars.next();
    let rest = chars.next();

    match (file, rank, rest) {
        (Some(f), Some(r), None) if ('a'..='h').contains(&f) && (r == '3' || r == '6') => {
            Ok(Some(field.to_string()))
        }
        _ => Err(FenError::InvalidEnPassant(field.to_string())),
    }
}
