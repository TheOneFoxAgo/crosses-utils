use crate::cell_type::*;
use crate::*;

pub struct GameBoardImpl<'a, B: GameBoard + ?Sized> {
    pub board: &'a mut B,
}
impl<B: GameBoard + ?Sized> GameBoardImpl<'_, B> {
    pub fn make_move(
        &mut self,
        index: BoardIndex<B>,
        player: EntryPlayer<BoardCell<B>>,
    ) -> Result<(), GameCoreError> {
        let entry = self.board.entry(index);
        match entry.get_type() {
            CellType::Empty(empty) => {
                if empty.is_active(player) {
                    entry.cross_out(player);
                    for adjacent in self.board.adjacent(index) {
                        let entry = self.board.entry(adjacent);
                        match entry.get_type() {
                            CellType::Empty(mut empty) => empty.activate(player),
                            CellType::Cross(mut cross) => cross.activate(player),
                            CellType::Filled(filled) => {
                                if filled.get_player() == player {
                                    if !filled.is_alive() {
                                        self.board.revive(adjacent, |cell_type| match cell_type {
                                            CellType::Empty(mut empty) => empty.activate(player),
                                            CellType::Cross(mut cross) => cross.activate(player),
                                            CellType::Filled(mut filled) => filled.set_alive(true),
                                            _ => {}
                                        });
                                        if let CellType::Cross(mut cross) =
                                            self.board.entry(index).get_type()
                                        {
                                            cross.set_anchor(true);
                                        }
                                        if let CellType::Filled(mut filled) =
                                            self.board.entry(adjacent).get_type()
                                        {
                                            filled.set_important(true);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(())
                } else {
                    Err(GameCoreError::OutOfReach)
                }
            }
            CellType::Cross(_cross) => {
                todo!()
            }
            CellType::Filled(_filled) => todo!(),
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
