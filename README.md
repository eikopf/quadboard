# Quadboard
A type-safe SIMD implementation of the [quadboard](https://www.chessprogramming.org/Quad-Bitboards) data structure, originally written as part of the [konig](https://github.com/eikopf/konig) engine.

## Example Usage

```rust
use quadboard::{QuadBoard, Nibble};

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
            todo!()
        }
    }
}

fn main() {
    let qb = QuadBoard::<Piece>::empty();

    qb.write(Piece { color: Color::White, kind: Kind::Queen }, 7);
    qb.write(Piece { color: Color::Black, kind: Kind::Pawn }, 3);
    qb.write(Piece { color: Color::White, kind: Kind::Bishop }, 25);
    qb.write(Piece { color: Color::Black, kind: Kind::Rook }, 34);
}
```
## TODOs
- Refactor checked functions to use an `Index` type (backed by a private enum, like in halfling)
- Add testing for all options.
- Move RawQuadBoard to a private module and reexport it.
