use crate::*;

pub struct GameBoardImpl<'a, B: GameBoard + ?Sized> {
    pub board: &'a mut B,
}
impl<B: GameBoard + ?Sized> GameBoardImpl<'_, B> {
    pub fn make_move(
        &mut self,
        index: BoardIndex<B>,
        player: CellPlayer<BoardCell<B>>,
    ) -> Result<(), GameCoreError> {
        let entry = self.board.entry(index);
        match entry.get_type() {
            CellType::Empty => {
                if entry.is_active(player) {
                    entry.set_type(CellType::Cross);
                    entry.set_player(player);
                    entry.unset_anchor();
                    self.board.adjacent(index).into_iter().for_each(|i| {
                        let entry = self.board.entry(i);
                        match entry.get_type() {
                            CellType::Empty => {
                                entry.activate(player);
                            }
                            CellType::Cross => {
                                entry.activate(player);
                            }
                            CellType::Filled => {
                                if entry.get_player() == player {
                                    if let CellState::Dead = entry.get_state() {
                                        self.board.traverse(i, Activate::<B> { player });
                                        self.board.entry(index).set_anchor();
                                        self.board.entry(i).set_state(CellState::NearAnchor);
                                    }
                                }
                            }
                            _ => {}
                        }
                    });
                    Ok(())
                } else {
                    Err(GameCoreError::OutOfReach)
                }
            }
            CellType::Cross => {
                todo!()
            }
            CellType::Filled => todo!(),
            CellType::Border => Err(GameCoreError::BorderHit),
        }
    }
    pub fn cancel_move(&self, _index: BoardIndex<B>) -> Result<(), GameCoreError> {
        unimplemented!();
    }
    pub fn init(&self) {
        unimplemented!();
    }
}

struct Activate<B: GameBoard + ?Sized> {
    player: CellPlayer<BoardCell<B>>,
}
impl<B: GameBoard + ?Sized> detail::Sealed for Activate<B> {}
impl<B: GameBoard + ?Sized> Strategy for Activate<B> {
    type Board = B;

    fn is_traversed(&self, cell: &BoardCell<Self::Board>) -> bool {
        cell.get_state() == CellState::Alive
    }

    fn process(
        &self,
        board: &mut Self::Board,
        index: BoardIndex<Self::Board>,
    ) -> core::ops::ControlFlow<()> {
        let entry = board.entry(index);
        match entry.get_type() {
            CellType::Empty => entry.activate(self.player),
            CellType::Cross if entry.get_player() != self.player => entry.activate(self.player),
            CellType::Filled => entry.set_state(CellState::Alive),
            _ => {}
        }
        core::ops::ControlFlow::Continue(())
    }
}
