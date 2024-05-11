# Quadboard
A type-safe SIMD implementation of the [quadboard](https://www.chessprogramming.org/Quad-Bitboards) data structure, originally written as part of the [konig](https://github.com/eikopf/konig) engine.

The stability of this crate is contingent on the stability of the `portable_simd` feature (tracking issue [#86656](https://github.com/rust-lang/rust/issues/86656)), though in the interim this could be worked around with cargo flags.

## Usage
Suppose you want to use a quadboard to represent a chessboard state in the usual way, with pieces defined as follows:

```rust
#[repr(u8)]
#[derive(Default)]
enum Color {
    #[default]
    Black = 0b0000,
    White = 0b1000,
}

#[repr(u8)]
#[derive(Default)]
enum Kind {
    #[default]
    None = 0b000,
    Pawn = 0b001,
    Rook = 0b010,
    Knight = 0b011,
    Bishop = 0b100,
    Queen = 0b101,
    King = 0b110,
}

#[derive(Default)]
struct Piece {
    color: Color,
    kind: Kind,
}
```

This particular encoding can be used to fit a piece into a nibble by just taking the bitwise AND of its fields, so we can define the following encoding:

```rust
// going from a Piece to a Nibble is fairly simple
impl From<Piece> for Nibble {
    fn from(value: Piece) -> Self {
        let nibble = (value.color as u8) & (value.kind as u8);
        unsafe { Nibble::new_unchecked(nibble) }
    }
}

// in reverse, we have a few more problems to deal with:
// 1. 0b0111 and 0b1111 don't correpond to specific pieces, and;
// 2. 0b1000 and 0b0000 map to the same piece.
//
// these are details that should encourage you to use a different
// encoding scheme –– there's surprising room for improvement
impl From<Nibble> for Piece {
    fn from(value: Nibble) -> Self {
        // if you trust your encoding, std::mem::transmute 
        // is probably the fastest decoding implementation

        let byte: u8 = value.get();

        // match against the significand of 
        // the nibble to get the color
        let color: Color = match byte >> 3 {
            0 => Color::Black,
            1 => Color::White,
            _ => unreachable!,
        };

        let kind: Kind = match byte & (!0b111) {
            // left as an exercise to the reader...
            _ => todo!(),
        }

        Self { color, kind }
    }
}
```

And that's it! This is the minimum required to use a `Quadboard<Piece>`, mostly with the `get` and `set` methods as well as their unsafe `_unchecked` equivalents.

```rust
// create a quadboard filled with Piece::default()
let mut qb = Quadboard::<Piece>::default();

// insert some pieces
qb.set(7u8.try_into().unwrap(), Piece { color: Color::White, kind: Kind::Queen });
qb.set(3u8.try_into().unwrap(), Piece { color: Color::Black, kind: Kind::Pawn });
qb.set(25u8.try_into().unwrap(), Piece { color: Color::White, kind: Kind::Bishop });
qb.set(34u8.try_into().unwrap(), Piece { color: Color::Black, kind: Kind::Rook });

// and read them out
assert_eq!(
    qb.get(7u8.try_into().unwrap()),
    Piece { color: Color::White, kind: Kind::Queen }
)

// a quadboard is defined at all indices at all times
assert_eq!(
    qb.get(0u8.try_into().unwrap()),
    Piece { color: Color::Black, kind: Kind::None }
)
```
