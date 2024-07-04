use crate::*;

type Player<E> = <<E as Engine>::Data as Data>::Player;
pub fn make_move<E: Engine>(
    engine: &mut E,
    index: E::Index,
    player: Player<E>,
) -> Result<(), EngineError> {
    let mut data = engine.get(index);
    match data.kind() {
        DataKind::Empty => {
            if !data.is_active(player) {
                return Err(EngineError::OutOfReach);
            }
            data.cross_out(player);
            engine.set(index, data);
            *engine.crosses_counter(player) += 1;
            *engine.moves_counter(player) -= 1;
            for adjacent_index in engine.adjacent(index) {
                let mut adjacent_data = engine.get(adjacent_index);
                match adjacent_data.kind() {
                    DataKind::Empty => {
                        try_activate(engine, &mut adjacent_data, player);
                    }
                    DataKind::Cross => {
                        if adjacent_data.player() != player {
                            try_activate(engine, &mut adjacent_data, player);
                        }
                    }
                    DataKind::Filled => {
                        if adjacent_data.player() == player && !adjacent_data.is_alive() {
                            engine.revive(adjacent_index, |engine, i| {
                                revive_strategy(engine, i, player)
                            });
                            data.set_anchor(true);
                            adjacent_data.set_important(true);
                        }
                    }
                    _ => {}
                }
                engine.set(adjacent_index, adjacent_data);
            }
        }
        DataKind::Cross => {
            if data.player() == player {
                return Err(EngineError::SelfFill);
            }
            if data.is_active(player) {
                let is_anchor = data.is_anchor();
                let previous_player = data.player();
                data.fill(player);
                engine.set(index, data);
                *engine.moves_counter(player) -= 1;
                *engine.crosses_counter(previous_player) -= 1;
                // firstly we are killing adjacent filled cells, if cross was anchor
                // if we cant find another cross
                if is_anchor {
                    for adjacent_index in engine.adjacent(index) {
                        let adjacent_data = engine.get(adjacent_index);
                        if adjacent_data.kind() == DataKind::Filled
                            && adjacent_data.player() == previous_player
                            && adjacent_data.is_important()
                        {
                            if let Some(new_anchor_index) = engine.search(adjacent_index) {
                                let mut new_anchor = engine.get(new_anchor_index);
                                new_anchor.set_anchor(true);
                                engine.set(new_anchor_index, new_anchor);
                                mark_adjacent_as_important(
                                    engine,
                                    new_anchor_index,
                                    previous_player,
                                );
                            } else {
                                engine.kill(adjacent_index, |engine, i| {
                                    kill_strategy(engine, i, previous_player)
                                });
                            }
                        }
                    }
                }
                // secondly we deactivating other cells
                for adjacent_index in engine.adjacent(index) {
                    let mut adjacent_data = engine.get(adjacent_index);
                    match adjacent_data.kind() {
                        DataKind::Empty => {
                            try_activate(engine, &mut adjacent_data, player);
                            if !is_activated(engine, adjacent_index, previous_player) {
                                try_deactivate(engine, &mut adjacent_data, previous_player);
                            }
                        }
                        DataKind::Cross => {
                            let adjacent_player = adjacent_data.player();
                            if adjacent_player != player {
                                try_activate(engine, &mut adjacent_data, player);
                            }
                            if adjacent_player != previous_player
                                && !is_activated(engine, adjacent_index, previous_player)
                            {
                                try_deactivate(engine, &mut adjacent_data, player);
                            }
                        }
                        _ => {}
                    }
                    engine.set(adjacent_index, adjacent_data);
                }
            }
        }
        DataKind::Filled => return Err(EngineError::DoubleFill),
        DataKind::Border => return Err(EngineError::BorderHit),
    };
    return Ok(());
}
pub fn cancel_move<E: Engine>(_engine: &mut E, _index: E::Index) -> Result<(), EngineError> {
    unimplemented!()
}
pub fn init<E: Engine>(_engine: &mut E) {
    unimplemented!()
}
/// Used with empty and cross cells. Activates cell and updates the counter
/// if target cell isn't active.
fn try_activate<E: Engine>(engine: &mut E, data: &mut E::Data, player: Player<E>) {
    if !data.is_active(player) {
        data.set_active(player, true);
        *engine.moves_counter(player) += 1;
    }
}
/// Used with empty and cross cells. Deactivates cell and updates the counter
/// if target cell is active.
fn try_deactivate<E: Engine>(engine: &mut E, data: &mut E::Data, player: Player<E>) {
    if data.is_active(player) {
        data.set_active(player, false);
        *engine.moves_counter(player) -= 1;
    }
}

fn is_activated<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) -> bool {
    engine
        .adjacent(index)
        .into_iter()
        .map(|i| engine.get(i))
        .find(|d| match d.kind() {
            DataKind::Cross if d.player() == player => true,
            DataKind::Filled if d.player() == player && d.is_alive() => true,
            _ => false,
        })
        .is_some()
}

fn revive_strategy<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) {
    let mut data = engine.get(index);
    match data.kind() {
        DataKind::Empty => {
            try_activate(engine, &mut data, player);
        }
        DataKind::Cross => {
            if data.player() != player {
                try_activate(engine, &mut data, player);
            }
        }
        DataKind::Filled => {
            if data.player() == player {
                data.set_alive(true)
            }
        }
        _ => {}
    }
    engine.set(index, data);
}
fn kill_strategy<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) {
    let mut data = engine.get(index);
    match data.kind() {
        DataKind::Empty => {
            if !is_activated(engine, index, player) {
                try_deactivate(engine, &mut data, player);
            }
        }
        DataKind::Cross => {
            if data.player() != player && !is_activated(engine, index, player) {
                try_deactivate(engine, &mut data, player);
            }
        }
        DataKind::Filled => {
            if data.player() == player {
                data.set_alive(false)
            }
        }
        _ => {}
    }
    engine.set(index, data);
}
fn mark_adjacent_as_important<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) {
    if let Some(adjacent_index) = engine.adjacent(index).into_iter().find(|i| {
        let d = engine.get(*i);
        d.kind() == DataKind::Filled && d.player() == player
    }) {
        let mut adjacent_data = engine.get(adjacent_index);
        adjacent_data.set_important(true);
        engine.set(adjacent_index, adjacent_data);
    }
}
