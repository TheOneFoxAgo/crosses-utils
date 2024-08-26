//! Importance-Based Traverse Strategy (IBTS)
//!
//! This module contains implementation of IBTS.
//! in form of [`IbtsBoard`] trait.
//! IBTS answers the question: when should we traverse a chain of filled cells?
//! The algorithm is very memory-efficient and allows you to greatly reduce the number of
//! chain traversals (By the end of the game, the savings only increases).
//! It works like this: if a new cross or a painted cell revives the chain of filled cells,
//! then the beginning of this chain and the “activator” are marked as “important”
//! (It should be noted a special case when a cell is filled in the absence of other filled
//! cells, then it and the cross are also marked as “important”). If an important cell is
//! removed, all adjacent important cells, if they have no other important neighbors, cease
//! to be important and in the case of filled cells the search for a new activator begins
//! (And, of course, if no such activator is found, the chain is killed).
use crate::base::{CellKind, GameBoard};

/// An implementation of IBTS.
/// It manages the importance and aliveness of cells.
pub trait IbtsBoard: GameBoard {
    /// Checks if the cell at given index is important.
    fn is_important(&self, index: Self::Index) -> bool;
    /// Sets new importance value to the cell at given index.
    fn set_important(&mut self, index: Self::Index, new: bool);
    /// Checks if the cell at given index is alive.
    fn is_alive(&self, index: Self::Index) -> bool;
    /// Sets new aliveness value to the cell at given index.
    fn set_alive(&mut self, index: Self::Index, new: bool);

    /// Revives the chain of filled cells (Sets their alive value to `true`)
    /// It's guaranteed that cell at given index whould be dead.
    fn revive(&mut self, index: Self::Index);
    /// Kills the chain of filled cells (Sets their alive value to `true`)
    /// It's guaranteed that cell at given index whould be alive.
    fn kill(&mut self, index: Self::Index);
    /// Searches for new activator. Should return `None` or index of new
    /// activator with adjacent filled cell that belongs to the searched chain.
    fn search(&mut self, index: Self::Index) -> Option<SearchResult<Self::Index>>;

    /// Should be called after changing cell at given index from
    /// [`CellKind::Empty`] to [`CellKind::Cross`].
    fn on_place_cross(&mut self, index: Self::Index) {
        revive_around(self, index, self.player(index));
    }
    /// Should be called after changing cell at given index from
    /// [`CellKind::Cross`] to [`CellKind::Filled`].
    /// Former player of cell should be passed as previous_player.
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
    /// Should be called after changing cell at given index from
    /// [`CellKind::Filled`] to [`CellKind::Cross`].
    /// Former player of cell should be passed as previous_player.
    fn on_remove_filled(&mut self, index: Self::Index, previous_player: Self::Player) {
        if self.is_important(index) {
            kill_around(self, index, previous_player);
        }
        revive_around(self, index, self.player(index));
    }
    /// Should be called after changing cell at given index from
    /// [`CellKind::Cross`] to [`CellKind::Empty`].
    /// Former player of cell should be passed as previous_player.
    fn on_remove_cross(&mut self, index: Self::Index, previous_player: Self::Player) {
        if self.is_important(index) {
            kill_around(self, index, previous_player);
        }
    }
}

/// A struct representing a search result. `cross` is the index of
/// found activator and `filled` is the index of adjacent filled cell
/// that belongs to chain that is being searched.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SearchResult<I> {
    pub filled: I,
    pub cross: I,
}

fn revive_around<M: IbtsBoard + ?Sized>(manager: &mut M, index: M::Index, player: M::Player) {
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
fn kill_around<M: IbtsBoard + ?Sized>(manager: &mut M, index: M::Index, player: M::Player) {
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
fn are_alive_filled_around<M: IbtsBoard + ?Sized>(
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
fn mark_adjacent_as_important<M: IbtsBoard + ?Sized>(
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
fn is_paired<M: IbtsBoard + ?Sized>(manager: &mut M, index: M::Index, player: M::Player) -> bool {
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
