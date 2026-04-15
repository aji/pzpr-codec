use puzzle_grid::array::{Array, ArrayBuffer, ArrayVec};

use crate::common::base27;

pub fn decode(s: &str) -> Result<ArrayVec<Cell>, String> {
    let (yinyang, s) = s.split_once('/').ok_or("missing '/'")?;
    let (cols, s) = s.split_once('/').ok_or("missing '/'")?;
    let (rows, gridstr) = s.split_once('/').ok_or("missing '/'")?;

    if yinyang != "yinyang" {
        return Err("puzzle type is not 'yinyang'".into());
    }
    let Ok(cols) = usize::from_str_radix(cols, 10) else {
        return Err("invalid columns".into());
    };
    let Ok(rows) = usize::from_str_radix(rows, 10) else {
        return Err("invalid rows".into());
    };
    if rows < 1 || cols < 1 {
        return Err(format!("invalid size: {rows},{cols}"));
    }

    Ok(base27::sb27be_to_ib3(gridstr.chars())?
        .into_iter()
        .take((rows * cols) as usize)
        .map(Cell::try_from_digit)
        .collect::<Result<ArrayVec<_>, _>>()?
        .reshape(rows, cols)
        .expect("reshape error"))
}

pub fn encode<B: ArrayBuffer<Item = Cell>>(board: &Array<B>) -> Result<String, String> {
    let grid = base27::ib3_to_sb27be(board.iter().map(Cell::to_digit))?;
    Ok(format!("yinyang/{}/{}/{grid}", board.cols(), board.rows(),))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Black,
    White,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

impl Cell {
    fn try_from_digit(x: u8) -> Result<Cell, String> {
        match x {
            0 => Ok(Cell::Empty),
            1 => Ok(Cell::White),
            2 => Ok(Cell::Black),
            _ => Err(format!("invalid digit: {x}")),
        }
    }

    fn to_digit(&self) -> u8 {
        match self {
            Cell::Empty => 0,
            Cell::White => 1,
            Cell::Black => 2,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Cell::*;

    // . . W B .
    // . W . W .
    // . . B B .
    // . . W . .
    const PUZZLE_ENCODED: &'static str = "yinyang/5/4/1ia0o10";
    const PUZZLE_DECODED: &[Cell] = [
        [Empty, Empty, White, Black, Empty],
        [Empty, White, Empty, White, Empty],
        [Empty, Empty, Black, Black, Empty],
        [Empty, Empty, White, Empty, Empty],
    ]
    .as_flattened();

    #[test]
    pub fn test_decode() {
        let puz = decode(PUZZLE_ENCODED).unwrap();

        assert_eq!(puz.rows(), 4);
        assert_eq!(puz.cols(), 5);

        for (a, b) in PUZZLE_DECODED.iter().zip(puz.iter()) {
            assert_eq!(a, b);
        }
    }

    #[test]
    pub fn test_encode() {
        let puz = Array::new(4, 5, PUZZLE_DECODED);
        assert_eq!(encode(&puz).unwrap(), PUZZLE_ENCODED)
    }
}
