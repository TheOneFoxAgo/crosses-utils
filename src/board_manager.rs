pub trait BoardManager: Sized {
    type Index: Copy;
    type Cell: Cell;
    fn get(&self, index: Self::Index) -> Self::Cell;
    fn set(&mut self, index: Self::Index, cell: Self::Cell);
    fn adjacent(&self, index: Self::Index) -> [Self::Index; 8];
    fn revive(&mut self, index: Self::Index, revive: impl FnMut(&mut Self, Self::Index));
    fn kill(&mut self, index: Self::Index, kill: impl FnMut(&mut Self, Self::Index));
    fn search(&mut self, index: Self::Index) -> Option<Self::Index>;
    fn moves_counter(&mut self, player: Player<Self>) -> &mut usize;
    fn crosses_counter(&mut self, player: Player<Self>) -> &mut usize;
    fn make_move(&mut self, index: Self::Index, player: Player<Self>) -> Result<(), EngineError> {
        let mut cell = self.get(index);
        match cell.kind() {
            CellKind::Empty => {
                if !cell.is_active(player) {
                    return Err(EngineError::OutOfReach);
                }
                cell.cross_out(player);
                *self.crosses_counter(player) += 1;
                *self.moves_counter(player) -= 1;
                cell.set_important(activate_around(self, index, player));
            }
            CellKind::Cross => {
                if cell.player() == player {
                    return Err(EngineError::SelfFill);
                }
                if cell.is_active(player) {
                    let was_important = cell.is_important();
                    let previous_player = cell.player();
                    cell.fill(player);
                    *self.moves_counter(player) -= 1;
                    *self.crosses_counter(previous_player) -= 1;
                    self.set(index, cell);
                    deactivate_around(self, index, previous_player, was_important);
                    cell.set_important(activate_around(self, index, player));
                }
            }
            CellKind::Filled => return Err(EngineError::DoubleFill),
            CellKind::Border => return Err(EngineError::BorderHit),
        };
        self.set(index, cell);
        Ok(())
    }
    fn cancel_move(&mut self, index: Self::Index, player: Player<Self>) -> Result<(), EngineError> {
        let mut cell = self.get(index);
        match cell.kind() {
            CellKind::Empty => return Err(EngineError::EmptyCancel),
            CellKind::Cross => {
                let was_important = cell.is_important();
                let previous_player = cell.player();
                cell.remove_cross();
                *self.crosses_counter(previous_player) -= 1;
                *self.moves_counter(previous_player) += 1;
                self.set(index, cell);
                deactivate_around(self, index, previous_player, was_important);
            }
            CellKind::Filled => {
                let was_important = cell.is_important();
                let previous_player = cell.player();
                cell.remove_fill(player);
                *self.moves_counter(previous_player) += 1;
                *self.crosses_counter(player) += 1;
                self.set(index, cell);
                deactivate_around(self, index, previous_player, was_important);
                cell.set_important(activate_around(self, index, player));
                restore_cross_activity(self, index, &mut cell);
            }
            CellKind::Border => return Err(EngineError::BorderHit),
        }
        self.set(index, cell);
        Ok(())
    }
}
pub type Player<E> = <<E as BoardManager>::Cell as Cell>::Player;
pub trait Cell: Copy {
    type Player: Copy + PartialEq;
    fn kind(self) -> CellKind;
    fn player(self) -> Self::Player;
    fn is_active(self, player: Self::Player) -> bool;
    fn set_active(&mut self, player: Self::Player, new: bool);
    fn is_important(self) -> bool;
    fn set_important(&mut self, new: bool);
    fn is_alive(self) -> bool;
    fn set_alive(&mut self, new: bool);

    fn cross_out(&mut self, player: Self::Player);
    fn fill(&mut self, player: Self::Player);
    fn remove_fill(&mut self, player: Self::Player);
    fn remove_cross(&mut self);
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CellKind {
    Empty,
    Cross,
    Filled,
    Border,
}
#[cfg_attr(feature = "std", derive(Error))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum EngineError {
    SelfFill,
    DoubleFill,
    BorderHit,
    OutOfReach,
    EmptyCancel,
}
/// Activates cells around given index. Revives filled cells.
/// It's fine to `set` cell only after call to this function.
#[must_use]
pub fn activate_around<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
) -> bool {
    let mut is_important = false;
    for adjacent_index in manager.adjacent(index) {
        let mut adjacent_cell = manager.get(adjacent_index);
        match adjacent_cell.kind() {
            CellKind::Empty => {
                try_activate(manager, &mut adjacent_cell, player);
            }
            CellKind::Cross => {
                if adjacent_cell.player() != player {
                    try_activate(manager, &mut adjacent_cell, player);
                }
            }
            CellKind::Filled => {
                if adjacent_cell.player() == player && !adjacent_cell.is_alive() {
                    manager.revive(adjacent_index, |manager, i| {
                        revive_strategy(manager, i, player)
                    });
                    is_important = true;
                    adjacent_cell.set_important(true);
                }
            }
            _ => {}
        }
        manager.set(adjacent_index, adjacent_cell);
    }
    is_important
}
/// Deactivates cells around given index. Kills filled cells.
/// Requires to `set` new state before calling.
fn deactivate_around<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
    was_important: bool,
) {
    if was_important {
        deactivate_filled_around(manager, index, player)
    }
    deactivate_remaining_around(manager, index, player);
}
/// Kills filled cells around index.
/// Requires to `set` new state before calling.
fn deactivate_filled_around<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) {
    for adjacent_index in manager.adjacent(index) {
        let mut adjacent_cell = manager.get(adjacent_index);
        if adjacent_cell.kind() == CellKind::Filled
            && adjacent_cell.player() == player
            && adjacent_cell.is_important()
        {
            if let Some(new_important_index) = manager.search(adjacent_index) {
                let mut new_important = manager.get(new_important_index);
                new_important.set_important(true);
                manager.set(new_important_index, new_important);
                mark_adjacent_as_important(manager, new_important_index, player);
            } else {
                manager.kill(adjacent_index, |manager, i| {
                    kill_strategy(manager, i, player)
                });
                if !is_paired(manager, adjacent_index, player) {
                    adjacent_cell.set_important(false);
                    manager.set(adjacent_index, adjacent_cell);
                }
            }
        }
    }
}
/// Deactivates cells around given index.
/// Requires to `set` new state and call to deactivate_filled_around if needed before calling.
fn deactivate_remaining_around<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
) {
    for adjacent_index in manager.adjacent(index) {
        let mut adjacent_cell = manager.get(adjacent_index);
        match adjacent_cell.kind() {
            CellKind::Empty => {
                if !is_activated(manager, adjacent_index, player) {
                    try_deactivate(manager, &mut adjacent_cell, player);
                }
            }
            CellKind::Cross => {
                let adjacent_player = adjacent_cell.player();
                if adjacent_player != player && !is_activated(manager, adjacent_index, player) {
                    try_deactivate(manager, &mut adjacent_cell, player);
                }
            }
            _ => {}
        }
        manager.set(adjacent_index, adjacent_cell);
    }
}
fn restore_cross_activity<M: BoardManager>(manager: &mut M, index: M::Index, cell: &mut M::Cell) {
    for adjacent_index in manager.adjacent(index) {
        let adjacent_cell = manager.get(adjacent_index);
        let adjacent_kind = adjacent_cell.kind();
        if adjacent_kind == CellKind::Cross
            || (adjacent_kind == CellKind::Filled && adjacent_cell.is_alive())
        {
            let adjacent_player = adjacent_cell.player();
            if adjacent_player != cell.player() {
                cell.set_active(adjacent_player, true)
            }
        }
    }
}
fn mark_adjacent_as_important<M: BoardManager>(
    manager: &mut M,
    index: M::Index,
    player: Player<M>,
) {
    if let Some(adjacent_index) = manager.adjacent(index).into_iter().find(|i| {
        let d = manager.get(*i);
        d.kind() == CellKind::Filled && d.player() == player
    }) {
        let mut adjacent_cell = manager.get(adjacent_index);
        adjacent_cell.set_important(true);
        manager.set(adjacent_index, adjacent_cell);
    }
}
fn is_activated<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) -> bool {
    manager
        .adjacent(index)
        .into_iter()
        .map(|i| manager.get(i))
        .any(|d| match d.kind() {
            CellKind::Cross => d.player() == player,
            CellKind::Filled => d.player() == player && d.is_alive(),
            _ => false,
        })
}
fn is_paired<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) -> bool {
    manager
        .adjacent(index)
        .into_iter()
        .map(|i| manager.get(i))
        .any(|d| match d.kind() {
            CellKind::Cross | CellKind::Filled => d.player() == player && d.is_important(),
            _ => false,
        })
}
fn revive_strategy<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) {
    let mut cell = manager.get(index);
    match cell.kind() {
        CellKind::Empty => {
            try_activate(manager, &mut cell, player);
        }
        CellKind::Cross => {
            if cell.player() != player {
                try_activate(manager, &mut cell, player);
            }
        }
        CellKind::Filled => {
            if cell.player() == player {
                cell.set_alive(true)
            }
        }
        _ => {}
    }
    manager.set(index, cell);
}
fn kill_strategy<M: BoardManager>(manager: &mut M, index: M::Index, player: Player<M>) {
    let mut cell = manager.get(index);
    match cell.kind() {
        CellKind::Empty => {
            if !is_activated(manager, index, player) {
                try_deactivate(manager, &mut cell, player);
            }
        }
        CellKind::Cross => {
            if cell.player() != player && !is_activated(manager, index, player) {
                try_deactivate(manager, &mut cell, player);
            }
        }
        CellKind::Filled => {
            if cell.player() == player {
                cell.set_alive(false)
            }
        }
        _ => {}
    }
    manager.set(index, cell);
}
/// Used with empty and cross cells. Activates cell and updates the counter
/// if target cell isn't active.
fn try_activate<M: BoardManager>(manager: &mut M, cell: &mut M::Cell, player: Player<M>) {
    if !cell.is_active(player) {
        cell.set_active(player, true);
        *manager.moves_counter(player) += 1;
    }
}
/// Used with empty and cross cells. Deactivates cell and updates the counter
/// if target cell is active.
fn try_deactivate<M: BoardManager>(manager: &mut M, cell: &mut M::Cell, player: Player<M>) {
    if cell.is_active(player) {
        cell.set_active(player, false);
        *manager.moves_counter(player) -= 1;
    }
}
