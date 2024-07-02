use core::cell::Cell;
use core::ops::ControlFlow;
pub trait Engine {
    type Index: Copy;
    type Data: Data;
    fn cell(&self, index: Self::Index) -> &Cell<Self::Data>;
    fn adjacent(&self, index: Self::Index) -> [Self::Index; 8];
    fn revive(&self, index: Self::Index, revive: impl Fn(Self::Index));
    fn kill(&self, index: Self::Index, kill: impl Fn(Self::Index));
    fn search(
        &self,
        index: Self::Index,
        search: impl Fn(Self::Index) -> ControlFlow<Self::Index>,
    ) -> Option<Self::Index>;
    fn make_move(
        &mut self,
        index: Self::Index,
        player: <Self::Data as Data>::Player,
    ) -> Result<(), GameCoreError>;
    fn cancel_move(&mut self, index: Self::Index) -> Result<(), GameCoreError>;
    fn init(&mut self);
}

pub trait Data: Copy {
    type Player: Copy + PartialEq;
    fn kind(self) -> DataKind;
    fn player(self) -> Self::Player;
    fn previous_player(self) -> Self::Player;
    fn is_active(self) -> bool;
    fn set_active(self, new: bool) -> Self;
    fn is_important(self) -> bool;
    fn set_important(self, new: bool) -> Self;
    fn is_alive(self) -> bool;
    fn set_alive(self, new: bool) -> Self;

    fn cross_out(self, player: Self::Player) -> Self;
    fn fill(self, player: Self::Player) -> Self;
    fn remove_fill(self, player: Self::Player) -> Self;
    fn remove_cross(self, player: Self::Player) -> Self;
}
pub enum DataKind {
    Empty,
    Cross,
    Filled,
    Border,
}

pub enum GameCoreError {
    SelfFill,
    DoubleFill,
    BorderHit,
    OutOfReach,
}

#[cfg(test)]
mod tests {}
