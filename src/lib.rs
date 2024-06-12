mod game_board_impl;
use game_board_impl::GameBoardImpl;

pub trait GameBoard {
    type Index: Copy;
    type Entry: BoardEntry;
    fn entry(&mut self, index: Self::Index) -> &mut Self::Entry;
    fn adjacent(&self, index: Self::Index) -> [Self::Index; 8];
    fn traverse<S: Strategy>(&mut self, idx: Self::Index, strategy: S) -> Option<Self::Index>;
    fn make_move(
        &mut self,
        index: Self::Index,
        player: CellPlayer<Self::Entry>,
    ) -> Result<(), GameCoreError> {
        GameBoardImpl { board: self }.make_move(index, player)
    }
    fn cancel_move(&mut self, index: Self::Index) -> Result<(), GameCoreError> {
        GameBoardImpl { board: self }.cancel_move(index)
    }
    fn init(&mut self) {
        GameBoardImpl { board: self }.init()
    }
}
type BoardIndex<B> = <B as GameBoard>::Index;
type BoardCell<B> = <B as GameBoard>::Entry;

pub trait BoardEntry {
    type Player: Copy + PartialEq;
    fn get_type(&self) -> CellType;
    fn set_type(&mut self, new_type: CellType);
    fn get_player(&self) -> Self::Player;
    fn set_player(&mut self, player: Self::Player);
    fn is_active(&self, player: Self::Player) -> bool;
    fn activate(&mut self, player: Self::Player);
    fn deactivate(&mut self, player: Self::Player);
    fn is_anchor(&self) -> bool;
    fn set_anchor(&mut self);
    fn unset_anchor(&mut self);
    fn get_previous_player(&self) -> Self::Player;
    fn set_previous_player(&mut self, player: Self::Player);
    fn get_state(&self) -> CellState;
    fn set_state(&mut self, new_state: CellState);
}
pub enum CellType {
    Empty,
    Cross,
    Filled,
    Border,
}

#[derive(PartialEq, Eq)]
pub enum CellState {
    Dead,
    Alive,
    NearAnchor,
    Between,
}
type CellPlayer<C> = <C as BoardEntry>::Player;

pub trait Strategy: detail::Sealed {
    type Board: GameBoard + ?Sized;
    fn is_traversed(&self, cell: &BoardCell<Self::Board>) -> bool;
    fn process(
        &self,
        board: &mut Self::Board,
        index: BoardIndex<Self::Board>,
    ) -> core::ops::ControlFlow<()>;
}
mod detail {
    pub trait Sealed {}
}

pub enum GameCoreError {
    SelfFill,
    BorderHit,
    OutOfReach,
}

#[cfg(test)]
mod tests {}
