use core::ops::ControlFlow;
mod engine_impl;

pub trait Engine: Sized {
    type Index: Copy;
    type Data: Data;
    fn get(&self, index: Self::Index) -> Self::Data;
    fn set(&mut self, index: Self::Index, data: Self::Data);
    fn adjacent(&self, index: Self::Index) -> [Self::Index; 8];
    fn revive(&mut self, index: Self::Index, revive: impl Fn(&mut Self, Self::Index));
    fn kill(&mut self, index: Self::Index, kill: impl Fn(&mut Self, Self::Index));
    fn search(
        &self,
        index: Self::Index,
        search: impl Fn(Self::Index) -> ControlFlow<Self::Index>,
    ) -> Option<Self::Index>;
    fn moves_counter(&mut self, player: Player<Self>) -> &mut usize;
    fn crosses_counter(&mut self, player: Player<Self>) -> &mut usize;
    fn make_move(&mut self, index: Self::Index, player: Player<Self>) -> Result<(), EngineError> {
        engine_impl::make_move(self, index, player)
    }
    fn cancel_move(&mut self, index: Self::Index) -> Result<(), EngineError> {
        engine_impl::cancel_move(self, index)
    }
    fn init(&mut self) {
        engine_impl::init(self)
    }
}
type Player<E> = <<E as Engine>::Data as Data>::Player;

pub trait Data: Copy {
    type Player: Copy + PartialEq;
    fn kind(self) -> DataKind;
    fn player(self) -> Self::Player;
    fn previous_player(self) -> Self::Player;
    fn is_active(self, player: Self::Player) -> bool;
    fn set_active(&mut self, player: Self::Player, new: bool);
    fn is_anchor(self) -> bool;
    fn set_anchor(&mut self, new: bool);
    fn is_important(self) -> bool;
    fn set_important(&mut self, new: bool);
    fn is_alive(self) -> bool;
    fn set_alive(&mut self, new: bool);

    fn cross_out(&mut self, player: Self::Player);
    fn fill(&mut self, player: Self::Player);
    fn remove_fill(&mut self, player: Self::Player);
    fn remove_cross(&mut self, player: Self::Player);
}
pub enum DataKind {
    Empty,
    Cross,
    Filled,
    Border,
}

pub enum EngineError {
    SelfFill,
    DoubleFill,
    BorderHit,
    OutOfReach,
}

#[cfg(test)]
mod tests {}
