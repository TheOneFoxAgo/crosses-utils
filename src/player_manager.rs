use core::{fmt::Display, ops::IndexMut};
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PlayerManager<S: IndexMut<usize, Output = Option<LoseData>>>
{
    remaining_moves: usize,
    max_number_of_moves: usize,
    current_player_idx: usize,
    total_number_of_players: usize,
    losers: S,
    current_move_idx: usize,
    game_result: GameState,
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LoseData {
    pub move_idx: usize,
    pub remaining_moves: usize,
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum GameState {
    Ongoing,
    Ended(GameOver)
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum GameOver {
    Win(usize),
    Draw,
}
impl Display for GameOver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GameOver::Win(winner) => write!(f, "game was won by player: {}", winner),
            GameOver::Draw => write!(f, "game ended in a draw"),
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for GameOver {}

impl<S> PlayerManager<S>
where
    S: IndexMut<usize, Output = Option<LoseData>>,
{
    pub fn new(max_number_of_moves: usize, total_number_of_players: usize, losers: S) -> Self {
        Self {
            remaining_moves: max_number_of_moves,
            max_number_of_moves,
            current_player_idx: 0,
            total_number_of_players,
            losers,
            current_move_idx: 0,
            game_result: GameState::Ongoing,
        }
    }
    pub fn remaining_moves(&self) -> usize {
        self.remaining_moves
    }
    pub fn max_number_of_moves(&self) -> usize {
        self.max_number_of_moves
    }
    pub fn current_player_idx(&self) -> usize {
        self.current_player_idx
    }
    pub fn total_number_of_players(&self) -> usize {
        self.total_number_of_players
    }
    pub fn losers(&self) -> &S {
        &self.losers
    }
    pub fn current_move_idx(&self) -> usize {
        self.current_move_idx
    }
    pub fn game_result(&self) -> GameState {
        self.game_result
    }
    pub fn next_player(
        &mut self,
        is_ran_out_of_moves: impl Fn(usize) -> bool,
        is_ran_out_of_crosses: impl Fn(usize) -> bool,
    ) -> Result<usize, GameOver> {
        if let GameState::Ended(game_over) = self.game_result {
            return Err(game_over);
        }
        self.remaining_moves -= 1;
        let mut should_change_player = false;
        let mut should_check_everyone = false;
        if self.remaining_moves == 0 {
            should_change_player = true
        } else if is_ran_out_of_moves(self.current_player_idx) {
            self.losers[self.current_player_idx] = Some(LoseData {
                move_idx: self.current_move_idx,
                remaining_moves: self.remaining_moves,
            });
            should_change_player = true;
            should_check_everyone = true
        }
        if should_change_player {
            self.check_if_other_players_have_lost(
                should_check_everyone,
                is_ran_out_of_moves,
                is_ran_out_of_crosses,
            )?;
            match self.count_not_losers() {
                0 => self.game_result = GameState::Ended(GameOver::Draw),
                1 => {
                    self.game_result = GameState::Ended(GameOver::Win(
                        (0..self.total_number_of_players)
                            .find(|idx| self.losers[*idx].is_none())
                            .unwrap(),
                    ))
                }
                _ => {
                    self.current_player_idx = self.next_player_idx();
                    self.remaining_moves = self.max_number_of_moves;
                }
            }
        }
        self.current_move_idx += 1;
        Ok(self.current_player_idx)
    }
    pub fn previous_player(&mut self, previous_player_idx: usize) {
        self.current_move_idx -= 1;
        if let Some(LoseData {
            move_idx: _,
            remaining_moves,
        }) = self.losers[previous_player_idx]
        {
            self.remaining_moves = remaining_moves;
            let mut loser_idx = previous_player_idx;
            loop {
                if matches!(
                    self.losers[loser_idx],
                    Some(LoseData {
                        move_idx, 
                        remaining_moves: _
                    }) if move_idx == self.current_move_idx )
                {
                    self.losers[loser_idx] = None
                }
                if loser_idx == self.current_player_idx {
                    break;
                } else {
                    loser_idx = (loser_idx + 1) % self.total_number_of_players;
                }
            }
        } else {
            self.remaining_moves = if self.remaining_moves == 0 
            {self.max_number_of_moves} else {self.remaining_moves - 1};
        }
        self.current_player_idx = previous_player_idx;
    }
    fn check_if_other_players_have_lost(
        &mut self, 
        check_all: bool,
        is_ran_out_of_moves: impl Fn(usize) -> bool,
        is_ran_out_of_crosses: impl Fn(usize) -> bool,
    ) -> Result<(), GameOver> {
        let mut maybe_not_losers = self.count_not_losers();
        for delta in 1..self.total_number_of_players {
            let not_loser_idx = (self.current_player_idx + delta) % self.total_number_of_players;
            if self.losers[not_loser_idx].is_none() {
                {
                    if is_ran_out_of_crosses(not_loser_idx)
                    {
                        self.losers[not_loser_idx] = Some(LoseData {
                            move_idx: self.current_move_idx,
                            remaining_moves: 0,
                        });
                        maybe_not_losers -= 1;
                    } else if is_ran_out_of_moves(not_loser_idx)
                    {
                        if maybe_not_losers > 1 {
                            self.losers[not_loser_idx] = Some(LoseData {
                                move_idx: self.current_move_idx,
                                remaining_moves: 0,
                            });
                        } else {
                            break;
                        }
                    } else if !check_all {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
    fn count_not_losers(&self) -> usize {
        (0..self.total_number_of_players)
            .filter(|idx| self.losers[*idx].is_none())
            .count()
    }
    fn next_player_idx(&self) -> usize {
        for delta in 1..self.total_number_of_players {
            let not_loser_idx = (self.current_player_idx + delta) % self.total_number_of_players;
            if self.losers[not_loser_idx].is_none() {
                return not_loser_idx;
            }
        }
        0 //FIXME: change to unreachable, when sure that code is correct
    }
}
