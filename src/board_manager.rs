use core::write;
use core::{fmt::Display, ops::ControlFlow};

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
    fn is_active(&self, index: Self::Index, player: Self::Player) -> bool;
    fn set_active(&mut self, index: Self::Index, player: Self::Player, new: bool);
    fn is_important(&self, index: Self::Index) -> bool;
    fn set_important(&mut self, index: Self::Index, new: bool);
    fn is_alive(&self, index: Self::Index) -> bool;
    fn set_alive(&mut self, index: Self::Index, new: bool);
    /// Changes the type of cell to [`CellKind::Cross`].
    /// This function is only called for cells of type
    /// [`CellKind::Empty`]
    /// Kind should change to [`CellKind::Cross`]
    /// Player is changed to given player.
    /// Activity isn't changed, except the activity of a given player,
    /// that may be changed.
    /// Cell shouldn't be important.
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Empty*/;
    /// let player = /*some player*/;
    /// cell.cross_out(player);
    /// assert_eq!(cell.kind(), CellKind::Cross);
    /// assert_eq!(cell.player(), player);
    /// assert!(!cell.is_important());
    /// ```
    fn cross_out(&mut self, index: Self::Index, player: Self::Player);
    /// Changes the type of cell to [`CellKind::Filled`].
    /// This function is only called for cells of type
    /// [`CellKind::Cross`]
    /// Kind should change to [`CellKind::Filled`]
    /// Player is changed to given player.
    /// Cell should be alive and not important.
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Cross*/;
    /// let player = /*some player*/;
    /// cell.fill(player);
    /// assert_eq!(cell.kind(), CellKind::Filled);
    /// assert_eq!(cell.player(), player);
    /// assert!(cell.is_alive());
    /// assert!(!cell.is_important());
    /// ```
    fn fill(&mut self, index: Self::Index, player: Self::Player);
    /// Changes the type of cell to [`CellKind::Cross`].
    /// This function is only called for cells of type
    /// [`CellKind::Filled`]
    /// Kind should change to [`CellKind::Cross`]
    /// Player is changed to given player.
    /// *Cell should be deactivated for all players!*
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Cross*/;
    /// let player = /*some player*/;
    /// cell.remove_fill(player);
    /// assert_eq!(cell.kind(), CellKind::Cross);
    /// assert_eq!(cell.player(), player);
    /// ```
    fn remove_fill(&mut self, index: Self::Index, player: Self::Player);
    /// Changes the type of cell to [`CellKind::Empty`].
    /// This function is only called for cells of type
    /// [`CellKind::Cross`]
    /// Kind should change to [`CellKind::Empty`]
    /// Player is changed to given player.
    /// Activity isn't changed, except the activity of a player that owns this cross.
    /// It should be activated.
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Empty*/;
    /// let player = /*some player*/;
    /// cell.remove_cross();
    /// assert_eq!(cell.kind(), CellKind::Empty);
    /// assert!(cell.is_active(player));
    /// ```
    fn remove_cross(&mut self, index: Self::Index);
    /// Generic traverse function. Revive, kill and search are derived from it.
    /// If you define revive, kill and search manually, traverse wouldn't be used.
    /// `index` is the index of filled cell, from where traverse should start.
    /// `action` is closure, that need to be applied to every filled cell in
    /// chain and to every adjacent cell.
    fn traverse(
        &mut self,
        index: Self::Index,
        action: impl FnMut(&mut Self, Self::Index) -> ControlFlow<Self::Index, ()>,
    ) -> Option<Self::Index>;
    /// Traverse that revives the chain of filled cells.
    fn revive(&mut self, index: Self::Index, mut revive: impl FnMut(&mut Self, Self::Index)) {
        self.traverse(index, |manager, action| {
            revive(manager, action);
            ControlFlow::Continue(())
        });
    }
    /// Traverse that kills the chain of filled cells.
    fn kill(&mut self, index: Self::Index, mut kill: impl FnMut(&mut Self, Self::Index)) {
        self.traverse(index, |manager, action| {
            kill(manager, action);
            ControlFlow::Continue(())
        });
    }
    /// Traverse that searches for cross adjacent to the chain of filled cells.
    fn search(
        &mut self,
        index: Self::Index,
        search: impl FnMut(&mut Self, Self::Index) -> ControlFlow<Self::Index, ()>,
    ) -> Option<Self::Index> {
        self.traverse(index, search)
    }
    /// Makes move to the board.
    /// `index` - is the position on the board.
    /// `player` - is player that should make the move.
    /// If this function detects that move is incorrect, it will return `Err(BoardError)`
    fn make_move(&mut self, index: Self::Index, player: Self::Player) -> Result<(), BoardError> {
        match self.kind(index) {
            CellKind::Empty => {
                if !self.is_active(index, player) {
                    return Err(BoardError::OutOfReach);
                }
                self.cross_out(index, player);
                let should_set_important = activate_around(self, index, player);
                self.set_important(index, should_set_important);
            }
            CellKind::Cross => {
                if self.player(index) == player {
                    return Err(BoardError::SelfFill);
                }
                if !self.is_active(index, player) {
                    return Err(BoardError::OutOfReach);
                }
                let was_important = self.is_important(index);
                let previous_player = self.player(index);
                self.fill(index, player);
                deactivate_around(self, index, previous_player, was_important);
                let mut important = false;
                if !are_alive_filled_around(self, index, player) {
                    important = true;
                    mark_adjacent_as_important(self, index, player, CellKind::Cross);
                }
                let should_set_important = activate_around(self, index, player) || important;
                self.set_important(index, should_set_important);
            }
            CellKind::Filled => return Err(BoardError::DoubleFill),
            CellKind::Border => return Err(BoardError::BorderHit),
        };
        Ok(())
    }
    /// The inverse of `make_move`
    /// `index` - is position of move, that should be cancelled.
    /// `get_player` - is getter of player, that placed cross.
    /// this closure is only called when cell at `index` is filled.
    fn cancel_move(
        &mut self,
        index: Self::Index,
        mut get_player: impl FnMut() -> Self::Player,
    ) -> Result<(), BoardError> {
        match self.kind(index) {
            CellKind::Empty => return Err(BoardError::EmptyCancel),
            CellKind::Cross => {
                let was_important = self.is_important(index);
                let previous_player = self.player(index);
                self.remove_cross(index);
                deactivate_around(self, index, previous_player, was_important);
            }
            CellKind::Filled => {
                let player = get_player();
                let was_important = self.is_important(index);
                let previous_player = self.player(index);
                self.remove_fill(index, player);
                deactivate_around(self, index, previous_player, was_important);
                let should_set_important = activate_around(self, index, player);
                self.set_important(index, should_set_important);
            }
            CellKind::Border => return Err(BoardError::BorderHit),
        }
        Ok(())
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
/// Activates cell around given cell, if needed.
/// If you need to restore activity of cells on the board,
/// apply this function to every cell.
pub fn init<M: BoardManager + ?Sized>(manager: &mut M, index: M::Index, player: M::Player) {
    if manager.kind(index) == CellKind::Cross {
        let _ = activate_around(manager, index, player);
    }
}
/// Activates cells around given index. Revives filled cells.
/// It's fine to `set` cell only after call to this function.
#[must_use]
fn activate_around<M: BoardManager + ?Sized>(
    manager: &mut M,
    index: M::Index,
    player: M::Player,
) -> bool {
    let mut is_important = false;
    for i in manager.adjacent(index) {
        match manager.kind(index) {
            CellKind::Empty | CellKind::Cross => {
                manager.set_active(i, player, true);
            }
            CellKind::Filled => {
                if manager.player(i) == player && !manager.is_alive(i) {
                    manager.revive(i, |manager, j| revive_strategy(manager, j, player));
                    is_important = true;
                    manager.set_important(i, true);
                }
            }
            _ => {}
        }
    }
    is_important
}
/// Deactivates cells around given index. Kills filled cells.
/// Requires to `set` new state before calling.
fn deactivate_around<M: BoardManager + ?Sized>(
    manager: &mut M,
    index: M::Index,
    player: M::Player,
    was_important: bool,
) {
    if was_important {
        deactivate_filled_around(manager, index, player)
    }
    deactivate_remaining_around(manager, index, player);
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
/// Kills filled cells around index.
/// Requires to `set` new state before calling.
fn deactivate_filled_around<M: BoardManager + ?Sized>(
    manager: &mut M,
    index: M::Index,
    player: M::Player,
) {
    for i in manager.adjacent(index) {
        if manager.kind(i) == CellKind::Filled
            && manager.player(i) == player
            && manager.is_important(i)
        {
            if let Some(new_important_index) =
                manager.search(i, |manager, i| search_strategy(manager, i, player))
            {
                manager.set_important(i, true);
                mark_adjacent_as_important(manager, new_important_index, player, CellKind::Filled);
            } else {
                manager.kill(i, |manager, i| kill_strategy(manager, i, player));
                if !is_paired(manager, i, player) {
                    manager.set_important(i, false);
                }
            }
        }
    }
}
/// Deactivates cells around given index.
/// Requires to `set` new state and call to deactivate_filled_around if needed before calling.
fn deactivate_remaining_around<M: BoardManager + ?Sized>(
    manager: &mut M,
    index: M::Index,
    player: M::Player,
) {
    for i in manager.adjacent(index) {
        match manager.kind(i) {
            CellKind::Empty | CellKind::Cross => {
                manager.set_active(i, player, false);
            }
            _ => {}
        }
    }
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
fn revive_strategy<M: BoardManager + ?Sized>(manager: &mut M, index: M::Index, player: M::Player) {
    if let CellKind::Filled = manager.kind(index) {
        if manager.player(index) == player {
            manager.set_alive(index, true);
            for i in manager.adjacent(index) {
                if let CellKind::Cross | CellKind::Empty = manager.kind(i) {
                    manager.set_active(i, player, true);
                }
            }
        }
    }
}
fn kill_strategy<M: BoardManager + ?Sized>(manager: &mut M, index: M::Index, player: M::Player) {
    if let CellKind::Filled = manager.kind(index) {
        if manager.player(index) == player {
            manager.set_alive(index, false);
            for i in manager.adjacent(index) {
                if let CellKind::Cross | CellKind::Empty = manager.kind(i) {
                    manager.set_active(i, player, false);
                }
            }
        }
    }
}
fn search_strategy<M: BoardManager + ?Sized>(
    manager: &mut M,
    index: M::Index,
    player: M::Player,
) -> ControlFlow<M::Index> {
    match manager.kind(index) {
        CellKind::Cross if manager.player(index) == player => ControlFlow::Break(index),
        _ => ControlFlow::Continue(()),
    }
}
