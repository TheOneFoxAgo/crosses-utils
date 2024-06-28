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
                                        self.revive(adjacent, player);
                                        self.if_cross(index, |mut c| c.set_anchor(true));
                                        self.if_filled(index, |mut f| f.set_important(true));
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
            CellType::Cross(cross) => {
                if cross.get_player() == player {
                    return Err(GameCoreError::SelfFill);
                }
                if cross.is_active(player) {
                    let _previous_player = cross.get_player();
                    // TODO: Finish deactivation (Wish me luck);
                    entry.fill(player);
                    for adjacent in self.board.adjacent(index) {
                        let entry = self.board.entry(adjacent);
                        match entry.get_type() {
                            CellType::Empty(mut empty) => empty.activate(player),
                            CellType::Cross(mut cross) => cross.activate(player),
                            CellType::Filled(filled) => {
                                if filled.get_player() == player {
                                    if !filled.is_alive() {
                                        self.revive(adjacent, player);
                                        self.if_filled(index, |mut f| f.set_important(true));
                                    }
                                }
                            }
                            CellType::Border => todo!(),
                        }
                    }
                    Ok(())
                } else {
                    Err(GameCoreError::OutOfReach)
                }
            }
            CellType::Filled(_) => Err(GameCoreError::DoubleFill),
            CellType::Border => Err(GameCoreError::BorderHit),
        }
    }
    pub fn cancel_move(&self, _index: BoardIndex<B>) -> Result<(), GameCoreError> {
        unimplemented!();
    }
    pub fn init(&self) {
        unimplemented!();
    }
    fn revive(&mut self, index: BoardIndex<B>, player: EntryPlayer<BoardCell<B>>) {
        self.board.revive(index, |cell_type| match cell_type {
            CellType::Empty(mut empty) => empty.activate(player),
            CellType::Cross(mut cross) => cross.activate(player),
            CellType::Filled(mut filled) => filled.set_alive(true),
            _ => {}
        });
    }
    fn _if_emtpy(
        &mut self,
        index: BoardIndex<B>,
        action: impl Fn(<BoardCell<B> as BoardEntry>::Empty),
    ) {
        if let CellType::Empty(emtpy) = self.board.entry(index).get_type() {
            action(emtpy);
        }
    }
    fn if_cross(
        &mut self,
        index: BoardIndex<B>,
        action: impl Fn(<BoardCell<B> as BoardEntry>::Cross),
    ) {
        if let CellType::Cross(cross) = self.board.entry(index).get_type() {
            action(cross);
        }
    }
    fn if_filled(
        &mut self,
        index: BoardIndex<B>,
        action: impl Fn(<BoardCell<B> as BoardEntry>::Filled),
    ) {
        if let CellType::Filled(filled) = self.board.entry(index).get_type() {
            action(filled);
        }
    }
}
