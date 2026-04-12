use crate::{
    grid::{Grid, Gridlike},
    variety::common::base27,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Black,
    White,
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

pub fn encode(board: &Grid<Cell>) -> Result<String, String> {
    let grid = base27::ib3_to_sb27be(board.cells().map(Cell::to_digit))?;
    Ok(format!(
        "yinyang/{}/{}/{grid}",
        board.shape().cols(),
        board.shape().rows()
    ))
}

pub fn decode(s: &str) -> Result<Grid<Cell>, String> {
    let (yinyang, s) = s.split_once('/').ok_or("missing '/'")?;
    let (cols, s) = s.split_once('/').ok_or("missing '/'")?;
    let (rows, grid) = s.split_once('/').ok_or("missing '/'")?;

    if yinyang != "yinyang" {
        return Err("puzzle type is not 'yinyang'".into());
    }
    let Ok(cols) = isize::from_str_radix(cols, 10) else {
        return Err("invalid columns".into());
    };
    let Ok(rows) = isize::from_str_radix(rows, 10) else {
        return Err("invalid rows".into());
    };
    if rows < 1 || cols < 1 {
        return Err(format!("invalid size: {rows},{cols}"));
    }
    let grid = base27::sb27be_to_ib3(grid.chars())?
        .into_iter()
        .take((rows * cols) as usize)
        .map(Cell::try_from_digit)
        .collect::<Result<Grid<Cell>, _>>()?
        .reshape(rows, cols)?;

    Ok(grid)
}

#[cfg(test)]
mod test {
    use super::*;

    // . . W B .
    // . W . W .
    // . . B B .
    // . . W . .
    const PUZZLE: &'static str = "yinyang/5/4/1ia0o10";

    #[test]
    pub fn test_decode() {
        let puz = decode(PUZZLE).unwrap();

        assert_eq!(puz.shape().rows(), 4);
        assert_eq!(puz.shape().cols(), 5);

        assert_eq!(puz[0], Cell::Empty);
        assert_eq!(puz[1], Cell::Empty);
        assert_eq!(puz[2], Cell::White);
        assert_eq!(puz[3], Cell::Black);
        assert_eq!(puz[4], Cell::Empty);

        assert_eq!(puz[5], Cell::Empty);
        assert_eq!(puz[6], Cell::White);
        assert_eq!(puz[7], Cell::Empty);
        assert_eq!(puz[8], Cell::White);
        assert_eq!(puz[9], Cell::Empty);

        assert_eq!(puz[10], Cell::Empty);
        assert_eq!(puz[11], Cell::Empty);
        assert_eq!(puz[12], Cell::Black);
        assert_eq!(puz[13], Cell::Black);
        assert_eq!(puz[14], Cell::Empty);

        assert_eq!(puz[15], Cell::Empty);
        assert_eq!(puz[16], Cell::Empty);
        assert_eq!(puz[17], Cell::White);
        assert_eq!(puz[18], Cell::Empty);
        assert_eq!(puz[19], Cell::Empty);
    }

    #[test]
    pub fn test_encode() {
        use Cell::*;

        let grid_raw = [
            Empty, Empty, White, Black, Empty, Empty, White, Empty, White, Empty, Empty, Empty,
            Black, Black, Empty, Empty, Empty, White, Empty, Empty,
        ];
        let grid = grid_raw
            .into_iter()
            .collect::<Grid<Cell>>()
            .reshape(4, 5)
            .unwrap();

        assert_eq!(encode(&grid).unwrap(), PUZZLE)
    }
}
