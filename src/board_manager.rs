use core::fmt::Display;
use core::write;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait BoardManager {
    /// The type of indeces in the board
    type Index: Copy;

    type Adjacent: IntoIterator<Item = Self::Index>;

    type Player: Copy + PartialEq;

    /// Set `cell` at `index`
    /// Returns adjacent cells for some `index`
    /// # Example
    /// ```
    /// type B = /* some impl of BoardManager with Adjacent set to [usize; 8] */;
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

    fn is_important(&self, index: Self::Index) -> bool;
    fn set_important(&mut self, index: Self::Index, new: bool);
    fn is_alive(&self, index: Self::Index) -> bool;
    fn set_alive(&mut self, index: Self::Index, new: bool);

    fn revive(&mut self, index: Self::Index);
    fn kill(&mut self, index: Self::Index);
    fn search(&mut self, index: Self::Index) -> Option<SearchResult<Self::Index>>;

    fn on_place_cross(&mut self, index: Self::Index) {
        revive_around(self, index, self.player(index));
    }
    fn on_place_filled(&mut self, index: Self::Index, previous_player: Self::Player) {
        if self.is_important(index) {
            kill_around(self, index, previous_player);
        }
        if !are_alive_filled_around(self, index, self.player(index)) {
            self.set_important(index, true);
            mark_adjacent_as_important(self, index, self.player(index), CellKind::Cross);
        }
        revive_around(self, index, self.player(index));
    }
    fn on_remove_filled(&mut self, index: Self::Index, previous_player: Self::Player) {
        if self.is_important(index) {
            kill_around(self, index, previous_player);
        }
        revive_around(self, index, self.player(index));
    }
    fn on_remove_cross(&mut self, index: Self::Index, previous_player: Self::Player) {
        if self.is_important(index) {
            kill_around(self, index, previous_player);
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CellKind {
    Empty,
    Cross,
    Filled,
    Border,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum BoardError {
    SelfFill,
    DoubleFill,
    BorderHit,
    OutOfReach,
    EmptyCancel,
}
impl Display for BoardError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BoardError::SelfFill => write!(f, "can't fill cell with its own color"),
            BoardError::DoubleFill => write!(f, "can't fill filled cell"),
            BoardError::BorderHit => write!(f, "border hit"),
            BoardError::OutOfReach => write!(f, "cell is out of reach"),
            BoardError::EmptyCancel => write!(f, "can't cancel empty cell"),
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for BoardError {}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SearchResult<I> {
    filled: I,
    cross: I,
}
fn revive_around<M: BoardManager + ?Sized>(manager: &mut M, index: M::Index, player: M::Player) {
    for i in manager.adjacent(index) {
        if manager.kind(i) == CellKind::Filled
            && manager.player(i) == player
            && !manager.is_alive(i)
        {
            manager.revive(i);
            manager.set_important(i, true);
            manager.set_important(index, true);
        }
    }
}
fn kill_around<M: BoardManager + ?Sized>(manager: &mut M, index: M::Index, player: M::Player) {
    manager.set_important(index, false);
    for i in manager.adjacent(index) {
        if manager.kind(i) == CellKind::Filled
            && manager.player(i) == player
            && manager.is_alive(i)
            && manager.is_important(i)
        {
            manager.kill(i);
            if !is_paired(manager, i, player) {
                manager.set_important(i, false)
            }
        }
    }
}
fn are_alive_filled_around<M: BoardManager + ?Sized>(
    manager: &mut M,
    index: M::Index,
    player: M::Player,
) -> bool {
    manager
        .adjacent(index)
        .into_iter()
        .any(|i| match manager.kind(i) {
            CellKind::Filled => manager.player(i) == player && manager.is_alive(i),
            _ => false,
        })
}
fn mark_adjacent_as_important<M: BoardManager + ?Sized>(
    manager: &mut M,
    index: M::Index,
    player: M::Player,
    target: CellKind,
) {
    if let Some(important_index) = manager
        .adjacent(index)
        .into_iter()
        .find(|i| manager.kind(*i) == target && manager.player(*i) == player)
    {
        manager.set_important(important_index, true);
    }
}
fn is_paired<M: BoardManager + ?Sized>(
    manager: &mut M,
    index: M::Index,
    player: M::Player,
) -> bool {
    manager
        .adjacent(index)
        .into_iter()
        .any(|i| match manager.kind(i) {
            CellKind::Cross | CellKind::Filled => {
                manager.player(i) == player && manager.is_important(i)
            }
            _ => false,
        })
}
// /// Makes move to the board.
// /// `index` - is the position on the board.
// /// `player` - is player that should make the move.
// /// If this function detects that move is incorrect, it will return `Err(BoardError)`
// fn make_move(&mut self, index: Self::Index, player: Self::Player) -> Result<(), BoardError> {
//     match self.kind(index) {
//         CellKind::Empty => {
//             if !self.is_active(index, player) {
//                 return Err(BoardError::OutOfReach);
//             }
//             self.cross_out(index, player);
//             let should_set_important = activate_around(self, index, player);
//             self.set_important(index, should_set_important);
//         }
//         CellKind::Cross => {
//             if self.player(index) == player {
//                 return Err(BoardError::SelfFill);
//             }
//             if !self.is_active(index, player) {
//                 return Err(BoardError::OutOfReach);
//             }
//             let was_important = self.is_important(index);
//             let previous_player = self.player(index);
//             self.fill(index, player);
//             deactivate_around(self, index, previous_player, was_important);
//             let mut important = false;
//             if !are_alive_filled_around(self, index, player) {
//                 important = true;
//                 mark_adjacent_as_important(self, index, player, CellKind::Cross);
//             }
//             let should_set_important = activate_around(self, index, player) || important;
//             self.set_important(index, should_set_important);
//         }
//         CellKind::Filled => return Err(BoardError::DoubleFill),
//         CellKind::Border => return Err(BoardError::BorderHit),
//     };
//     Ok(())
// }
// /// The inverse of `make_move`
// /// `index` - is position of move, that should be cancelled.
// /// `get_player` - is getter of player, that placed cross.
// /// this closure is only called when cell at `index` is filled.
// fn cancel_move(
//     &mut self,
//     index: Self::Index,
//     mut get_player: impl FnMut() -> Self::Player,
// ) -> Result<(), BoardError> {
//     match self.kind(index) {
//         CellKind::Empty => return Err(BoardError::EmptyCancel),
//         CellKind::Cross => {
//             let was_important = self.is_important(index);
//             let previous_player = self.player(index);
//             self.remove_cross(index);
//             deactivate_around(self, index, previous_player, was_important);
//         }
//         CellKind::Filled => {
//             let player = get_player();
//             let was_important = self.is_important(index);
//             let previous_player = self.player(index);
//             self.remove_fill(index, player);
//             deactivate_around(self, index, previous_player, was_important);
//             let should_set_important = activate_around(self, index, player);
//             self.set_important(index, should_set_important);
//         }
//         CellKind::Border => return Err(BoardError::BorderHit),
//     }
//     Ok(())
// }
