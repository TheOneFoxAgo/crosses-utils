use crate::base::{CellKind, GameBoard};
use core::fmt::Display;

pub trait IbtsBoard: GameBoard {
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
