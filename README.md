# fenview

FEN (Forsyth-Edwards Notation) is the standard way to write down a chess
position as a single line of text, e.g.

    rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

It's compact and easy for engines to parse, but hard for a person to check
by eye. A typo in the piece placement field, a missing dash for castling
rights, or a stray digit is easy to miss when the whole position is one
dense string. `fenview` reads FEN records and renders each one as a board
diagram, reporting exactly what's wrong when a record doesn't parse.

## Usage

Pipe a FEN in on stdin:

    echo "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" | fenview

Or pass one or more files, each containing one FEN per line (blank lines
and lines starting with `#` are skipped):

    fenview openings.fen endgames.fen

Output for the starting position:

    == stdin:1 ==
    rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
      a b c d e f g h
    8 r n b q k b n r
    7 p p p p p p p p
    6 . . . . . . . .
    5 . . . . . . . .
    4 . . . . . . . .
    3 . . . . . . . .
    2 P P P P P P P P
    1 R N B Q K B N R

    to move: white
    castling: KQkq
    en passant: -
    halfmove clock: 0
    fullmove number: 1

If a record is malformed, `fenview` reports the source and line number
and keeps processing the rest of the input:

    $ echo "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN w KQkq - 0 1" | fenview
    fenview: stdin:1: rank 1 does not sum to 8 files (found 7)

The process exits with a non-zero status if any record failed to parse,
which makes it usable as a validation step, e.g. in a pre-commit hook for
a file of opening lines.

## Building

    cargo build --release

No third-party dependencies; the standard library is enough for parsing
and printing.

## Status

Early skeleton. Piece placement, active color, castling rights, en
passant target, halfmove clock, and fullmove number are all parsed and
validated. See the roadmap for what's still missing.
