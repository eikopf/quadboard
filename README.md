# Quadboard
A type-safe SIMD implementation of the [quadboard](https://www.chessprogramming.org/Quad-Bitboards) data structure, originally written as part of the [konig](https://github.com/eikopf/konig) engine.

The stability of this crate is contingent on the stability of the `portable_simd` feature (tracking issue [#86656](https://github.com/rust-lang/rust/issues/86656)), though in the interim this could be worked around with cargo flags.

## Example Usage

```rust
use quadboard::{Quadboard, Nibble};

/// The color of a piece, with values
/// chosen so that they map to the significand
/// of a nibble.
#[repr(u8)]
enum Color {
    Black = 0,
    White = 8,
}

/// The kind (i.e. type) of a piece.
#[repr(u8)]
enum Kind {
    None = 0,
    Pawn = 1,
    Rook = 2,
    Knight = 3,
    Bishop = 4,
    Queen = 5,
    King = 6,
}

/// A simple piece representation.
struct Piece {
    color: Color,
    kind: Kind,
}

// ... some impls have been cut for brevity ...

// here we define our encoding
impl From<Piece> for Nibble {
    fn from(value: Piece) -> Self {
        Nibble::try_from(value.color as u8 + value.kind as u8).unwrap()
    }
}

// in reverse, we need to consider undefined nibbles
impl TryFrom<Nibble> for Piece {
    // using unit for simplicity
    type Error = ();

    fn try_from(value: Nibble) -> Result<Self, Self::Error> {
        match value.get() {
            // left as an exercise to the reader...
            // with some bit twiddling, you might find
            // it easier to write this in a branchless way
            _ => todo!()
        }
    }
}

fn main() {
    let qb = Quadboard::<Piece>::empty();

    qb.write(Piece { color: Color::White, kind: Kind::Queen }, 7u8.try_into().unwrap());
    qb.write(Piece { color: Color::Black, kind: Kind::Pawn }, 3u8.try_into().unwrap());
    qb.write(Piece { color: Color::White, kind: Kind::Bishop }, 25u8.try_into().unwrap());
    qb.write(Piece { color: Color::Black, kind: Kind::Rook }, 34u8.try_into().unwrap());
}
```
## TODOs
- Add testing for all modules (including private modules).
- Publish to crates.io
