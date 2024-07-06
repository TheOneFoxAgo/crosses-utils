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
            *engine.crosses_counter(player) += 1;
            *engine.moves_counter(player) -= 1;
            data.set_important(activate_around(engine, index, player));
        }
        DataKind::Cross => {
            if data.player() == player {
                return Err(EngineError::SelfFill);
            }
            if data.is_active(player) {
                let was_important = data.is_important();
                let previous_player = data.player();
                data.fill(player);
                *engine.moves_counter(player) -= 1;
                *engine.crosses_counter(previous_player) -= 1;
                engine.set(index, data);
                deactivate_around(engine, index, previous_player, was_important);
                data.set_important(activate_around(engine, index, player));
            }
        }
        DataKind::Filled => return Err(EngineError::DoubleFill),
        DataKind::Border => return Err(EngineError::BorderHit),
    };
    engine.set(index, data);
    Ok(())
}
pub fn cancel_move<E: Engine>(
    engine: &mut E,
    index: E::Index,
    player: Player<E>,
) -> Result<(), EngineError> {
    let mut data = engine.get(index);
    match data.kind() {
        DataKind::Empty => return Err(EngineError::EmptyCancel),
        DataKind::Cross => {
            let was_important = data.is_important();
            let previous_player = data.player();
            data.remove_cross();
            *engine.crosses_counter(previous_player) -= 1;
            *engine.moves_counter(previous_player) += 1;
            engine.set(index, data);
            deactivate_around(engine, index, previous_player, was_important);
        }
        DataKind::Filled => {
            let was_important = data.is_important();
            let previous_player = data.player();
            data.remove_fill(player);
            *engine.moves_counter(previous_player) += 1;
            *engine.crosses_counter(player) += 1;
            engine.set(index, data);
            deactivate_around(engine, index, previous_player, was_important);
            data.set_important(activate_around(engine, index, player));
        }
        DataKind::Border => return Err(EngineError::BorderHit),
    }
    engine.set(index, data);
    Ok(())
}
/// Activates cells around given index. Revives filled cells.
/// It's fine to `set` cell only after call to this function.
#[must_use]
pub fn activate_around<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) -> bool {
    let mut is_special = false;
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
                    is_special = true;
                    adjacent_data.set_important(true);
                }
            }
            _ => {}
        }
        engine.set(adjacent_index, adjacent_data);
    }
    return is_special;
}
/// Deactivates cells around given index. Kills filled cells.
/// Requires to `set` new state before calling.
fn deactivate_around<E: Engine>(
    engine: &mut E,
    index: E::Index,
    player: Player<E>,
    was_important: bool,
) {
    if is_important {
        deactivate_filled_around(engine, index, previous_player)
    }
    deactivate_remaining_around(engine, index, previous_player);
}
/// Kills filled cells around index.
/// Requires to `set` new state before calling.
fn deactivate_filled_around<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) {
    for adjacent_index in engine.adjacent(index) {
        let mut adjacent_data = engine.get(adjacent_index);
        if adjacent_data.kind() == DataKind::Filled
            && adjacent_data.player() == player
            && adjacent_data.is_important()
        {
            if let Some(new_important_index) = engine.search(adjacent_index) {
                let mut new_important = engine.get(new_important_index);
                new_important.set_important(true);
                engine.set(new_important_index, new_important);
                mark_adjacent_as_important(engine, new_important_index, player);
            } else {
                engine.kill(adjacent_index, |engine, i| kill_strategy(engine, i, player));
                if !is_paired(engine, adjacent_index, player) {
                    adjacent_data.set_important(false);
                    engine.set(adjacent_index, adjacent_data);
                }
            }
        }
    }
}
/// Deactivates cells around given index.
/// Requires to `set` new state and call to deactivate_filled_around if needed before calling.
fn deactivate_remaining_around<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) {
    for adjacent_index in engine.adjacent(index) {
        let mut adjacent_data = engine.get(adjacent_index);
        match adjacent_data.kind() {
            DataKind::Empty => {
                if !is_activated(engine, adjacent_index, player) {
                    try_deactivate(engine, &mut adjacent_data, player);
                }
            }
            DataKind::Cross => {
                let adjacent_player = adjacent_data.player();
                if adjacent_player != player && !is_activated(engine, adjacent_index, player) {
                    try_deactivate(engine, &mut adjacent_data, player);
                }
            }
            _ => {}
        }
        engine.set(adjacent_index, adjacent_data);
    }
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
fn is_activated<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) -> bool {
    engine
        .adjacent(index)
        .into_iter()
        .map(|i| engine.get(i))
        .find(|d| match d.kind() {
            DataKind::Cross => d.player() == player,
            DataKind::Filled => d.player() == player && d.is_alive(),
            _ => false,
        })
        .is_some()
}
fn is_paired<E: Engine>(engine: &mut E, index: E::Index, player: Player<E>) -> bool {
    engine
        .adjacent(index)
        .into_iter()
        .map(|i| engine.get(i))
        .find(|d| match d.kind() {
            DataKind::Cross | DataKind::Filled => d.player() == player && d.is_important(),
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
