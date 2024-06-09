use crate::*;

pub struct GameBoardImpl<'a, B: GameBoard + ?Sized> {
    pub board: &'a mut B,
}
impl<B: GameBoard + ?Sized> GameBoardImpl<'_, B> {
    pub fn make_move(
        &self,
        _index: BoardIndex<B>,
        _player: CellPlayer<BoardCell<B>>,
    ) -> Result<(), GameCoreError> {
        unimplemented!();
    }
    pub fn cancel_move(&self, _index: BoardIndex<B>) -> Result<(), GameCoreError> {
        unimplemented!();
    }
    pub fn init(&self) {
        unimplemented!();
    }
}
