mod game_board_impl;
use game_board_impl::GameBoardImpl;

pub trait GameBoard {
    type Index: Copy;
    type Entry: BoardEntry;
    fn entry(&mut self, index: Self::Index) -> &mut Self::Entry;
    fn adjacent(&self, index: Self::Index) -> [Self::Index; 8];
    fn revive(&mut self, index: Self::Index, revive: impl Fn(EntryType<Self::Entry>));
    fn kill(&mut self, index: Self::Index, kill: impl Fn(EntryType<Self::Entry>));
    fn search(
        &mut self,
        index: Self::Index,
        search: impl Fn(EntryType<Self::Entry>) -> core::ops::ControlFlow<Self::Index>,
    ) -> Option<Self::Index>;
    fn make_move(
        &mut self,
        index: Self::Index,
        player: EntryPlayer<Self::Entry>,
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
type EntryType<E> = CellType<
    <E as BoardEntry>::Empty,
    <E as BoardEntry>::Cross,
    <E as BoardEntry>::Filled,
    <E as BoardEntry>::Player,
>;
type EntryPlayer<E> = <E as BoardEntry>::Player;

pub trait BoardEntry {
    type Player: Copy + PartialEq;
    type Empty: cell_type::Empty<Player = Self::Player>;
    type Cross: cell_type::Cross<Player = Self::Player>;
    type Filled: cell_type::Filled<Player = Self::Player>;
    fn get_type(&mut self) -> EntryType<Self>;
    fn cross_out(&mut self, player: Self::Player) -> Self::Cross;
    fn fill(&mut self, player: Self::Player) -> Self::Filled;
    fn remove_fill(&mut self, player: Self::Player) -> Self::Cross;
    fn remove_cross(&mut self, player: Self::Player) -> Self::Empty;
}

pub enum CellType<E, C, F, P>
where
    E: cell_type::Empty<Player = P>,
    C: cell_type::Cross<Player = P>,
    F: cell_type::Filled<Player = P>,
{
    Empty(E),
    Cross(C),
    Filled(F),
    Border,
}
pub mod cell_type {
    pub trait Empty {
        type Player;
        fn is_active(&self, player: Self::Player) -> bool;
        fn activate(&mut self, player: Self::Player);
        fn deactivate(&mut self, player: Self::Player);
    }
    pub trait Cross {
        type Player;
        fn get_player(&self) -> Self::Player;
        fn is_active(&self, player: Self::Player) -> bool;
        fn activate(&mut self, player: Self::Player);
        fn deactivate(&mut self, player: Self::Player);
        fn is_anchor(&self) -> bool;
        fn set_anchor(&mut self, is_anchor: bool);
    }
    pub trait Filled {
        type Player;
        fn get_player(&self) -> Self::Player;
        fn get_previous_player(&self) -> Self::Player;
        fn is_alive(&self) -> bool;
        fn set_alive(&mut self, is_alive: bool);
        fn is_important(&self) -> bool;
        fn set_important(&mut self, is_important: bool);
    }
}

pub enum GameCoreError {
    SelfFill,
    DoubleFill,
    BorderHit,
    OutOfReach,
}

#[cfg(test)]
mod tests {}
