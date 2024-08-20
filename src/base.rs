//! Base for utils
//!
//! This module defines a set of common structs and traits for
//! utils in this crate.
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The base trait for utils that work with game board.
pub trait GameBoard {
    /// The type of indeces in the board
    type Index: Copy;

    /// The type of adjacent indices
    type Adjacent: IntoIterator<Item = Self::Index>;

    /// The type of players in the board
    type Player: Copy + PartialEq;

    /// Returns indices of adjacent cells for some `index`
    fn adjacent(&mut self, index: Self::Index) -> Self::Adjacent;
    /// Returns the type of cell
    fn kind(&self, index: Self::Index) -> CellKind;
    /// Returns the player of cell
    fn player(&self, index: Self::Index) -> Self::Player;
}

/// A type representing kind of the cell.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CellKind {
    /// Empty cell
    Empty,
    /// Cell with cross in it
    Cross,
    /// Filled cell
    Filled,
    /// Border. Marker of "out of bounds".
    /// No operations would be performed with it.
    Border,
}
