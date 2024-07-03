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
                                let mut data = engine.get(i);
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
                                engine.set(i, data);
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
                let previous_player = data.player();
                data.fill(player);
                for adjacent_index in engine.adjacent(index) {
                    let mut adjacent_data = engine.get(adjacent_index);
                    match adjacent_data.kind() {
                        DataKind::Empty => todo!(),
                        DataKind::Cross => todo!(),
                        DataKind::Filled => todo!(),
                        DataKind::Border => todo!(),
                    }
                }
            }
        }
        DataKind::Filled => todo!(),
        DataKind::Border => return Err(EngineError::BorderHit),
    };
    engine.set(index, data);
    return Ok(());
}
pub fn cancel_move<E: Engine>(_engine: &mut E, _index: E::Index) -> Result<(), EngineError> {
    unimplemented!()
}
pub fn init<E: Engine>(_engine: &mut E) {
    unimplemented!()
}

fn try_activate<E: Engine>(engine: &mut E, data: &mut E::Data, player: Player<E>) {
    if !data.is_active(player) {
        data.set_active(player, true);
        *engine.moves_counter(player) += 1;
    }
}
fn _try_deactivate<E: Engine>(engine: &mut E, data: &mut E::Data, player: Player<E>) {
    if data.is_active(player) {
        data.set_active(player, false);
        *engine.moves_counter(player) -= 1;
    }
}
