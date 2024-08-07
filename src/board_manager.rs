use core::write;
use core::{fmt::Display, ops::ControlFlow};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait BoardManager: Sized {
    /// The type of indeces in the board
    type Index: Copy;
    /// The type of the cells in the board
    type Cell: Cell;
    /// Returns cell at `index`
    fn get(&self, index: Self::Index) -> Self::Cell;
    /// Set `cell` at `index`
    fn set(&mut self, index: Self::Index, cell: Self::Cell);
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
    fn adjacent(index: Self::Index) -> impl IntoIterator<Item = Self::Index>;
    fn add_to_moves_counter(&mut self, player: Player<Self>, amount: isize);
    fn add_to_crosses_counter(&mut self, player: Player<Self>, amount: isize);
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
    fn make_move(&mut self, index: Self::Index, player: Player<Self>) -> Result<(), BoardError> {
        let mut cell = self.get(index);
        match cell.kind() {
            CellKind::Empty => {
                if !cell.is_active(player) {
                    return Err(BoardError::OutOfReach);
                }
                cell.cross_out(player);
                self.add_to_crosses_counter(player, 1);
                self.add_to_moves_counter(player, -1);
                cell.set_important(activate_around(self, index, player));
            }
            CellKind::Cross => {
                if cell.player() == player {
                    return Err(BoardError::SelfFill);
                }
                if !cell.is_active(player) {
                    return Err(BoardError::OutOfReach);
                }
                let was_important = cell.is_important();
                let previous_player = cell.player();
                cell.fill(player);
                self.add_to_moves_counter(player, -1);
                self.add_to_crosses_counter(previous_player, -1);
                self.set(index, cell);
                deactivate_around(self, index, previous_player, was_important);
                let mut important = false;
                if !is_alive_filled_around(self, index, player) {
                    important = true;
                    mark_adjacent_as_important(self, index, player, CellKind::Cross);
                }
                cell.set_important(activate_around(self, index, player) || important);
            }
            CellKind::Filled => return Err(BoardError::DoubleFill),
            CellKind::Border => return Err(BoardError::BorderHit),
        };
        self.set(index, cell);
        Ok(())
    }
    /// The inverse of `make_move`
    /// `index` - is position of move, that should be cancelled.
    /// `get_player` - is getter of player, that placed cross.
    /// this closure is only called when cell at `index` is filled.
    fn cancel_move(
        &mut self,
        index: Self::Index,
        mut get_player: impl FnMut() -> Player<Self>,
    ) -> Result<(), BoardError> {
        let mut cell = self.get(index);
        match cell.kind() {
            CellKind::Empty => return Err(BoardError::EmptyCancel),
            CellKind::Cross => {
                let was_important = cell.is_important();
                let previous_player = cell.player();
                cell.remove_cross();
                self.add_to_crosses_counter(previous_player, -1);
                self.add_to_moves_counter(previous_player, 1);
                self.set(index, cell);
                deactivate_around(self, index, previous_player, was_important);
            }
            CellKind::Filled => {
                let player = get_player();
                let was_important = cell.is_important();
                let previous_player = cell.player();
                cell.remove_fill(player);
                self.add_to_moves_counter(previous_player, 1);
                self.add_to_crosses_counter(player, 1);
                self.set(index, cell);
                deactivate_around(self, index, previous_player, was_important);
                cell.set_important(activate_around(self, index, player));
            }
            CellKind::Border => return Err(BoardError::BorderHit),
        }
        self.set(index, cell);
        Ok(())
    }
}
pub type Player<E> = <<E as BoardManager>::Cell as Cell>::Player;
pub trait Cell: Copy {
    /// The type of player, that interacts with cell
    type Player: Copy + PartialEq;
    /// Returns the type of cell
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Empty*/;
    /// assert_eq!(cell.kind(), CellKind::Empty);
    /// cell.cross_out(/*some player*/);
    /// assert_eq!(cell.kind(), CellKind::Cross);
    /// // and so on
    /// ```
    fn kind(self) -> CellKind;
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
    fn player(self) -> Self::Player;
    /// Returns the activity of cell for given player
    /// This function is only called for cells of type
    /// [`CellKind::Empty`] and [`CellKind::Cross`]
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Empty*/;
    /// let player = /*some player*/;
    /// cell.set_active(player, true);
    /// assert!(cell.is_active(player));
    /// cell.set_active(player, false);
    /// assert!(!cell.is_active(player));
    /// ```
    fn is_active(self, player: Self::Player) -> bool;
    fn activate(&mut self, player: Self::Player) -> ActivationStatus;
    fn deactivate(&mut self, player: Self::Player) -> ActivationStatus;
    fn reset_activity(&mut self);
    /// Return the importance of cell.
    /// This function is only called for cells of type
    /// [`CellKind::Cross`] and [`CellKind::Filled`]
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Cross*/;
    /// cell.set_important(true);
    /// assert!(cell.is_important());
    /// ```    
    fn is_important(self) -> bool;
    /// Sets the importance of cell.
    /// This function is only called for cells of type
    /// [`CellKind::Cross`] and [`CellKind::Filled`]
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Cross*/;
    /// cell.set_important(true);
    /// assert!(cell.is_important());
    /// ```    
    fn set_important(&mut self, new: bool);
    /// Returns the aliveness of cell.
    /// This function is only called for cells of type
    /// [`CellKind::Filled`]
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Filled*/;
    /// cell.set_alive(true);
    /// assert!(cell.is_alive());
    /// ```    
    fn is_alive(self) -> bool;
    /// Sets the aliveness of cell.
    /// This function is only called for cells of type
    /// [`CellKind::Filled`]
    /// # Example
    /// ```
    /// let mut cell = /*some impl of Cell. Current type is Filled*/;
    /// cell.set_alive(true);
    /// assert!(cell.is_alive());
    /// ```    
    fn set_alive(&mut self, new: bool);
    fn is_overheated(self) -> bool;
    fn set_overheat(&mut self, new: bool);
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
    fn cross_out(&mut self, player: Self::Player);
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
    fn fill(&mut self, player: Self::Player);
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
    fn remove_fill(&mut self, player: Self::Player);
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
    fn remove_cross(&mut self);
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
pub enum ActivationStatus {
    Regular,
    Overheat,
    Zero,
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
pub fn init<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) {
    if manager.get(index).kind() == CellKind::Cross {
        let _ = activate_around(manager, index, player);
    }
}
/// Activates cells around given index. Revives filled cells.
/// It's fine to `set` cell only after call to this function.
#[must_use]
fn activate_around<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) -> bool {
    let mut is_important = false;
    for adjacent_index in M::adjacent(index) {
        let mut adjacent_cell = manager.get(adjacent_index);
        match adjacent_cell.kind() {
            CellKind::Empty => {
                try_activate(manager, &mut adjacent_cell, player);
            }
            CellKind::Cross => {
                if adjacent_cell.player() != player {
                    try_activate(manager, &mut adjacent_cell, player);
                }
            }
            CellKind::Filled => {
                if adjacent_cell.player() == player && !adjacent_cell.is_alive() {
                    manager.revive(adjacent_index, |manager, i| {
                        revive_strategy(manager, i, player)
                    });
                    is_important = true;
                    adjacent_cell.set_important(true);
                }
            }
            _ => {}
        }
        manager.set(adjacent_index, adjacent_cell);
    }
    is_important
}
/// Deactivates cells around given index. Kills filled cells.
/// Requires to `set` new state before calling.
fn deactivate_around<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
    was_important: bool,
) {
    if was_important {
        deactivate_filled_around(manager, index, player)
    }
    deactivate_remaining_around(manager, index, player);
}
fn is_alive_filled_around<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
) -> bool {
    M::adjacent(index)
        .into_iter()
        .map(|i| manager.get(i))
        .any(|d| match d.kind() {
            CellKind::Filled => d.player() == player && d.is_alive(),
            _ => false,
        })
}
/// Kills filled cells around index.
/// Requires to `set` new state before calling.
fn deactivate_filled_around<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) {
    for adjacent_index in M::adjacent(index) {
        let mut adjacent_cell = manager.get(adjacent_index);
        if adjacent_cell.kind() == CellKind::Filled
            && adjacent_cell.player() == player
            && adjacent_cell.is_important()
        {
            if let Some(new_important_index) = manager.search(adjacent_index, |manager, i| {
                search_strategy(manager, i, player)
            }) {
                let mut new_important = manager.get(new_important_index);
                new_important.set_important(true);
                manager.set(new_important_index, new_important);
                mark_adjacent_as_important(manager, new_important_index, player, CellKind::Filled);
            } else {
                manager.kill(adjacent_index, |manager, i| {
                    kill_strategy(manager, i, player)
                });
                if !is_paired(manager, adjacent_index, player) {
                    adjacent_cell.set_important(false);
                    manager.set(adjacent_index, adjacent_cell);
                }
            }
        }
    }
}
/// Deactivates cells around given index.
/// Requires to `set` new state and call to deactivate_filled_around if needed before calling.
fn deactivate_remaining_around<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
) {
    for adjacent_index in M::adjacent(index) {
        let mut adjacent_cell = manager.get(adjacent_index);
        match adjacent_cell.kind() {
            CellKind::Empty | CellKind::Cross => {
                try_deactivate(manager, &mut adjacent_cell, adjacent_index, player);
            }
            _ => {}
        }
        manager.set(adjacent_index, adjacent_cell);
    }
}
fn mark_adjacent_as_important<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
    target: CellKind,
) {
    if let Some(adjacent_index) = M::adjacent(index).into_iter().find(|i| {
        let d = manager.get(*i);
        d.kind() == target && d.player() == player
    }) {
        let mut adjacent_cell = manager.get(adjacent_index);
        adjacent_cell.set_important(true);
        manager.set(adjacent_index, adjacent_cell);
    }
}
fn is_paired<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) -> bool {
    M::adjacent(index)
        .into_iter()
        .map(|i| manager.get(i))
        .any(|d| match d.kind() {
            CellKind::Cross | CellKind::Filled => d.player() == player && d.is_important(),
            _ => false,
        })
}
fn revive_strategy<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) {
    let mut cell = manager.get(index);
    if let CellKind::Filled = cell.kind() {
        if cell.player() == player {
            cell.set_alive(true);
            manager.set(index, cell);
            for adjacent_index in M::adjacent(index) {
                let mut adjacent_cell = manager.get(adjacent_index);
                if let CellKind::Cross | CellKind::Empty = adjacent_cell.kind() {
                    try_activate(manager, &mut adjacent_cell, player);
                    manager.set(adjacent_index, adjacent_cell);
                }
            }
        }
    }
}
fn kill_strategy<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) {
    let mut cell = manager.get(index);
    if let CellKind::Filled = cell.kind() {
        if cell.player() == player {
            cell.set_alive(false);
            manager.set(index, cell);
            for adjacent_index in M::adjacent(index) {
                let mut adjacent_cell = manager.get(adjacent_index);
                if let CellKind::Cross | CellKind::Empty = adjacent_cell.kind() {
                    try_deactivate(manager, &mut adjacent_cell, adjacent_index, player);
                    manager.set(adjacent_index, adjacent_cell);
                }
            }
        }
    }
}
fn search_strategy<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
) -> ControlFlow<M::Index> {
    let cell = manager.get(index);
    match cell.kind() {
        CellKind::Cross if cell.player() == player => ControlFlow::Break(index),
        _ => ControlFlow::Continue(()),
    }
}
/// Used with empty and cross cells. Activates cell and updates the counter
/// if target cell isn't active.
fn try_activate<M: BoardManager>(manager: &mut M, cell: &mut M::Cell, player: Player<M>) {
    if !cell.is_active(player) {
        manager.add_to_moves_counter(player, 1);
    }
    if !(cell.kind() == CellKind::Cross && player == cell.player())
        && cell.activate(player) == ActivationStatus::Overheat
    {
        cell.set_overheat(true)
    }
}
/// Used with empty and cross cells. Deactivates cell and updates the counter
/// if target cell is active.
fn try_deactivate<M: BoardManager>(
    manager: &mut M,
    cell: &mut M::Cell,
    index: M::Index,
    player: Player<M>,
) {
    if cell.is_active(player) {
        debug_assert!(
            !(cell.kind() == CellKind::Cross && cell.player() == player),
            "Cross shouldn't be active for its own player"
        );
        if cell.deactivate(player) == ActivationStatus::Zero {
            if cell.is_overheated() {
                rebuild_activity(manager, index, cell);
                if cell.is_active(player) {
                    manager.add_to_moves_counter(player, -1);
                }
            } else {
                manager.add_to_moves_counter(player, -1);
            }
        }
    }
}
fn rebuild_activity<M: BoardManager>(manager: &mut M, index: M::Index, cell: &mut M::Cell) {
    cell.reset_activity();
    cell.set_overheat(false);
    for adjacent_index in M::adjacent(index) {
        let adjacent_cell = manager.get(adjacent_index);
        let adjacent_kind = adjacent_cell.kind();
        if adjacent_kind == CellKind::Cross
            || (adjacent_kind == CellKind::Filled && adjacent_cell.is_alive())
        {
            let adjacent_player = adjacent_cell.player();
            if !(cell.kind() == CellKind::Cross && adjacent_player == cell.player())
                && cell.activate(adjacent_player) == ActivationStatus::Overheat
            {
                cell.set_overheat(true)
            }
        }
    }
}
