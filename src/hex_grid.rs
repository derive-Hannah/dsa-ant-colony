use std::{
    collections::VecDeque,
    fmt::{self, Display},
    iter,
};

/// Bi-direction-expandable hexagonal grid
/// Implimented using double-ended queues
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexGrid<CellType: Clone + Default> {
    y_offset: i32,
    grid: VecDeque<(i32, VecDeque<CellType>)>,
}

impl<CellType: Clone + Default> HexGrid<CellType> {
    /// Creates a new hex grid of size 0 at (0, 0)
    pub fn new() -> Self {
        Self {
            y_offset: 0,
            grid: VecDeque::new(),
        }
    }

    /// Creates a new hex grid parallelagram of specified size filled by clones of elm
    /// Assumes x_left <= x_right and y_bottom <= y_top
    ///    A O O B
    ///   O O O O
    ///  C O O D
    /// A is at (x_left, y_top)
    /// B is at (x_right, y_top)
    /// C is at (x_left, y_bottom)
    /// D is at (x_right, y_bottom)
    pub fn from_bounds_parallelagram(
        elm: CellType,
        x_left: i32,
        x_right: i32,
        y_bottom: i32,
        y_top: i32,
    ) -> Self {
        Self {
            y_offset: y_bottom,
            grid: iter::repeat_n(
                {
                    (
                        x_left,
                        iter::repeat_n(elm, (x_right - x_left + 1) as usize).collect(),
                    )
                },
                (y_top - y_bottom + 1) as usize,
            )
            .collect(),
        }
    }

    /// Creates a new reversed hex grid parallelagram of specified size filled by clones of elm
    /// Assumes bottom_x_left <= bottom_x_right and y_bottom <= y_top
    ///  A O O B
    ///   O O O O
    ///    C O O D
    /// Let height = y_top - y_bottom
    /// A is at (bottom_x_left - height, y_top)
    /// B is at (bottom_x_right - height, y_top)
    /// C is at (bottom_x_left, y_bottom)
    /// D is at (bottom_x_right, y_bottom)
    pub fn from_bounds_rparallelagram(
        elm: CellType,
        bottom_x_left: i32,
        bottom_x_right: i32,
        y_bottom: i32,
        y_top: i32,
    ) -> Self {
        Self {
            y_offset: y_bottom,
            grid: iter::repeat_n(
                { iter::repeat_n(elm, (bottom_x_right - bottom_x_left + 1) as usize).collect() },
                (y_top - y_bottom + 1) as usize,
            )
            .enumerate()
            .map(|(idx, row)| (bottom_x_left - (idx as i32), row))
            .collect(),
        }
    }

    /// Creates a new hex grid hexagon of specified radius filled by clones of elm
    ///    D O C
    ///   O O O O
    ///  E O A O B
    ///   O O O O
    ///    F O G
    /// A is at (x, y)
    /// B is at (x + radius, y)
    /// C is at (x, y + radius)
    /// D is at (x - radius, y + radius)
    /// E is at (x - radius, y)
    /// F is at (x, y - radius)
    /// G is at (x + radius, y - radius)
    pub fn from_bounds_hexagon(elm: CellType, x: i32, y: i32, radius: i32) -> Self {
        Self {
            y_offset: y - radius,
            grid: (-radius.abs()..=radius.abs())
                .map(|y_off| {
                    (
                        (x - y_off).max(0),
                        iter::repeat_n(elm.clone(), (2 * radius + 1 - y_off.abs()) as usize)
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    /// Get the value at the designated position, or None if it doesn't exist
    pub fn get(&self, x: i32, y: i32) -> Option<&CellType> {
        self.grid
            .get((y - self.y_offset) as usize)
            .and_then(|(x_offset, row)| row.get((x - *x_offset) as usize))
    }

    /// Sets the cell at the position to contain the value provided
    /// If the position is out of bounds, grow the grid to accomodate
    pub fn set(&mut self, x: i32, y: i32, value: CellType) {
        // Get row or extend y to fit
        let real_y = y - self.y_offset;
        let (x_offset, row) = if real_y < 0 {
            self.y_offset += real_y;
            self.grid.reserve(real_y.abs() as usize);
            for _ in 0..real_y.abs() {
                self.grid
                    .push_front((0, VecDeque::from([CellType::default()])));
            }
            &mut self.grid[0]
        } else if real_y as usize >= self.grid.len() {
            self.grid.extend(iter::repeat_n(
                (0, VecDeque::from([CellType::default()])),
                real_y as usize - self.grid.len() + 1,
            ));
            &mut self.grid[real_y as usize]
        } else {
            &mut self.grid[real_y as usize]
        };

        // Get cell or extend x to fit
        let real_x = x - *x_offset;
        let cell = if real_x < 0 {
            *x_offset += real_x;
            row.reserve(real_x.abs() as usize);
            for _ in 0..real_x.abs() {
                row.push_front(CellType::default());
            }
            &mut row[0]
        } else if real_x as usize >= row.len() {
            row.extend(iter::repeat_n(
                CellType::default(),
                real_x as usize - row.len() + 1,
            ));
            &mut row[real_x as usize]
        } else {
            &mut row[real_x as usize]
        };

        *cell = value
    }
}

impl<CellType: Clone + Default + Display> Display for HexGrid<CellType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x_left, x_right) = self.grid.iter().fold((i32::MAX, i32::MIN), |acc, elm| {
            (acc.0.min(elm.0), acc.1.max(elm.0 + elm.1.len() as i32 - 1))
        });
        let (y_bottom, y_top) = (self.y_offset, self.y_offset + self.grid.len() as i32 - 1);
        let max_y_width = y_bottom.to_string().len().max(y_top.to_string().len());
        let mut cell_width = self.grid.iter().fold(0, |acc, elm| {
            acc.max(
                elm.1
                    .iter()
                    .fold(0, |acc, elm| acc.max(elm.to_string().len())),
            )
        });
        if cell_width.is_multiple_of(2) {
            cell_width += 1;
        }
        let cell_offset = cell_width + 3;
        let hex_offset = cell_offset / 2;

        fn pad(mut s: String, l: usize, back: bool) -> String {
            if back {
                s.push_str(&" ".repeat(l - s.len()));
                s
            } else {
                let mut out = " ".repeat(l - s.len());
                out.push_str(&s);
                out
            }
        }
        let mut indent = self.grid.len() as i32;
        for (y_off, (row_x_left, row)) in self.grid.iter().enumerate().rev() {
            indent -= 1;
            write!(
                f,
                "{} | {}",
                pad((y_bottom + y_off as i32).to_string(), max_y_width, false),
                pad(
                    String::new(),
                    (indent * hex_offset as i32 + (row_x_left - x_left) * cell_offset as i32).max(0)
                        as usize,
                    true
                )
            )?;
            for cell in row {
                write!(f, "[{}] ", pad(cell.to_string(), cell_width, true))?;
            }
            writeln!(f)?;
        }
        write!(
            f,
            "{}",
            pad(
                String::new(),
                max_y_width + (indent * hex_offset as i32).max(0) as usize + 2,
                false
            ),
        )?;
        for _ in x_left..=x_right {
            write!(f, "{}", "-".repeat(cell_offset))?;
        }
        writeln!(f)?;
        write!(
            f,
            "{}",
            pad(
                String::new(),
                max_y_width + (indent * hex_offset as i32).max(0) as usize + 2,
                false
            ),
        )?;
        for num in x_left..=x_right {
            write!(f, "{} ", pad(num.to_string(), cell_width + 2, true))?;
        }
        writeln!(f)?;
        Ok(())
    }
}
