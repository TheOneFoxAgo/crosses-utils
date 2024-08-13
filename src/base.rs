#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait GameBoard {
    /// The type of indeces in the board
    type Index: Copy;

    type Adjacent: IntoIterator<Item = Self::Index>;

    type Player: Copy + PartialEq;

    /// Set `cell` at `index`
    /// Returns adjacent cells for some `index`
    /// # Example
    /// ```
    /// type B = /* some impl of IbtsBoard with Adjacent set to [usize; 8] */;
    /// // The board looks something like this:
    /// // [ 0, 1, 2,
    /// //   3, 4, 5,
    /// //   6, 7, 8 ]
    /// let mut adjacent: Vec<usize> = B::adjacent(4).into_iter().collect();
    /// adjacent.sort()
    /// assert_eq!(adjacent, [0, 1, 2, 3, 5, 6, 7, 8]);
    /// ```
    fn adjacent(&mut self, index: Self::Index) -> Self::Adjacent;
    /// Returns the type of cell
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Empty*/;
    /// assert_eq!(cell.kind(), CellKind::Empty);
    /// cell.cross_out(/*some player*/);
    /// assert_eq!(cell.kind(), CellKind::Cross);
    /// // and so on
    /// ```
    fn kind(&self, index: Self::Index) -> CellKind;
    /// Returns the player of cell
    /// This function is only called for cells of type
    /// `CellKind::Cross` and `CellKind::filled`
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Empty*/;
    /// let player = /*some player*/;
    /// cell.cross_out(player);
    /// assert_eq!(cell.player(), player);
    /// let other_player = /*some other player*/;
    /// cell.fill(other_player);
    /// assert_eq!(cell.player(), other_player);
    /// ```
    fn player(&self, index: Self::Index) -> Self::Player;
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CellKind {
    Empty,
    Cross,
    Filled,
    Border,
}
