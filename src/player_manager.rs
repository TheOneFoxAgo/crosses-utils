//! Manager of players' data
//!
//! This module defines [`PlayerManager`] struct, which is designed to help with player state management.
//! The main methods are [`advance`] and [`reverse`]. [`advance`] changes the state as it is expected after the player performs a move.
//! It keeps track of current move of the game, remaining moves, changes the player if necessary, the [`reverse`] method does the same thing,
//! but in reverse as the name suggests.
//!
//! [`advance`]: PlayerManager::advance
//! [`reverse`]: PlayerManager::reverse

use core::{fmt::Display, ops::IndexMut};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Helper structure to track players' state during game.
/// `S` - is type of storage. It can be Vec or simple array.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PlayerManager<S: IndexMut<usize, Output = Option<LoseData>>> {
    pub remaining_moves: usize,
    pub max_moves: usize,
    pub current_player: usize,
    pub max_players: usize,
    pub current_move: usize,
    pub game_state: GameState,
    pub losers: S,
}
impl<S> PlayerManager<S>
where
    S: IndexMut<usize, Output = Option<LoseData>>,
{
    /// Creates new [`PlayerManager`]. `remaining_moves` is set to `max_moves`,
    /// `current_player` and `current_move` are set to `0`, `game_state` is
    /// [`GameState::Ongoing`].
    /// `loosers` should be able to work with indeces from `0..max_players`
    /// if the "size" of `loosers` is less than `max_players` surtain
    /// methods will panic unexpectedly. Also all players shouldn't be
    /// loosers initially (All values are `None`).
    /// # Example
    /// ```
    /// # use crosses_utils::player_manager::*;
    /// let pm = PlayerManager::new(4, 4, [None; 4]);
    /// ```
    pub fn new(max_moves: usize, max_players: usize, losers: S) -> Self {
        debug_assert!((0..max_players).all(|i| losers[i] == None));
        Self {
            remaining_moves: max_moves,
            max_moves,
            current_player: 0,
            max_players,
            losers,
            current_move: 0,
            game_state: GameState::Ongoing,
        }
    }
    /// Advances state of the game. It decrements number of moves,
    /// changes current_player if needed, etc.
    /// `is_ran_ot_of_...` - are functions, that need to tell player_manager
    /// about player at given index. They are used for marking players as
    /// loosers.
    /// # Panics
    /// Panics if the game is over.
    /// # Example
    /// ```
    /// # use crosses_utils::player_manager::*;
    /// let mut pm = PlayerManager::new(4, 2, [None; 2]);
    /// assert_eq!(
    ///     (
    ///         pm.remaining_moves,
    ///         pm.current_player,
    ///         pm.current_move,
    ///         pm.game_state,
    ///     ),
    ///     (4, 0, 0, GameState::Ongoing)
    /// );
    /// pm.advance(|_| false, |_| false);
    /// assert_eq!(
    ///     (pm.remaining_moves, pm.current_player, pm.current_move),
    ///     (3, 0, 1)
    /// );
    /// for _ in 0..3 {
    ///     pm.advance(|_| false, |_| false);
    /// }
    /// assert_eq!(
    ///     (pm.remaining_moves, pm.current_player, pm.current_move),
    ///     (4, 1, 4)
    /// );
    /// pm.advance(|_| true, |_| false);
    /// assert_eq!(pm.game_state, GameState::Ended(GameOver::Win(0)));
    /// ```
    pub fn advance(
        &mut self,
        is_ran_out_of_moves: impl Fn(usize) -> bool,
        is_ran_out_of_crosses: impl Fn(usize) -> bool,
    ) {
        if self.game_state != GameState::Ongoing {
            panic!("Game has already ended, can't advance further!")
        }
        self.remaining_moves -= 1;
        let mut should_change_player = false;
        let mut should_check_everyone = false;
        if self.remaining_moves == 0 {
            should_change_player = true
        } else if is_ran_out_of_moves(self.current_player) {
            self.losers[self.current_player] = Some(LoseData {
                move_index: self.current_move,
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
            );
            match self.count_not_losers() {
                0 => self.game_state = GameState::Ended(GameOver::Draw),
                1 => {
                    self.game_state = GameState::Ended(GameOver::Win(
                        (0..self.max_players)
                            .find(|idx| self.losers[*idx].is_none())
                            .unwrap(),
                    ))
                }
                _ => {
                    self.current_player = self.next_player_idx();
                    self.remaining_moves = self.max_moves;
                }
            }
        }
        self.current_move += 1;
    }
    /// Reverses state of the game. It increments number of moves,
    /// changes current_player if needed, etc.
    /// To reverse the game state, we need to know what player
    /// was making the move. We can't get this info only from
    /// state of game_manager, so we must ask BoardManager for that.
    /// # Panics:
    /// if current_move is 0, the function will panic.
    /// # Example
    /// ```
    /// # use crosses_utils::player_manager::*;
    /// let mut pm = PlayerManager::new(4, 2, [None; 2]);
    /// pm.advance(|_| true, |_| false);
    /// assert_eq!(
    ///     (
    ///         pm.remaining_moves,
    ///         pm.current_player,
    ///         pm.current_move,
    ///         pm.game_state,
    ///     ),
    ///     (3, 0, 1, GameState::Ended(GameOver::Win(1)))
    /// );
    /// pm.reverse(0);
    /// assert_eq!(
    ///     (
    ///         pm.remaining_moves,
    ///         pm.current_player,
    ///         pm.current_move,
    ///         pm.game_state,
    ///     ),
    ///     (4, 0, 0, GameState::Ongoing)
    /// );
    /// ```
    pub fn reverse(&mut self, player: usize) {
        self.current_move -= 1;
        self.game_state = GameState::Ongoing;
        if let Some(LoseData {
            move_index: _,
            remaining_moves,
        }) = self.losers[player]
        {
            self.remaining_moves = remaining_moves;
            let mut loser_idx = player;
            loop {
                if let Some(LoseData {
                    move_index,
                    remaining_moves: _,
                }) = self.losers[loser_idx]
                {
                    if move_index == self.current_move {
                        self.losers[loser_idx] = None
                    }
                }
                if loser_idx == self.current_player {
                    break;
                } else {
                    loser_idx = (loser_idx + 1) % self.max_players;
                }
            }
        } else if self.remaining_moves == self.max_moves {
            self.remaining_moves = 0
        }
        self.current_player = player;
        self.remaining_moves += 1;
    }
    fn check_if_other_players_have_lost(
        &mut self,
        check_all: bool,
        is_ran_out_of_moves: impl Fn(usize) -> bool,
        is_ran_out_of_crosses: impl Fn(usize) -> bool,
    ) {
        let mut maybe_not_losers = self.count_not_losers();
        for delta in 1..self.max_players {
            let not_loser_idx = (self.current_player + delta) % self.max_players;
            if self.losers[not_loser_idx].is_none() {
                {
                    if is_ran_out_of_crosses(not_loser_idx) {
                        self.losers[not_loser_idx] = Some(LoseData {
                            move_index: self.current_move,
                            remaining_moves: 0,
                        });
                        maybe_not_losers -= 1;
                    } else if is_ran_out_of_moves(not_loser_idx) {
                        if maybe_not_losers > 1 {
                            self.losers[not_loser_idx] = Some(LoseData {
                                move_index: self.current_move,
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
    }
    fn count_not_losers(&self) -> usize {
        (0..self.max_players)
            .filter(|idx| self.losers[*idx].is_none())
            .count()
    }
    fn next_player_idx(&self) -> usize {
        for delta in 1..self.max_players {
            let not_loser_idx = (self.current_player + delta) % self.max_players;
            if self.losers[not_loser_idx].is_none() {
                return not_loser_idx;
            }
        }
        unreachable!()
    }
}
/// An information about losers. `move_index` is the index of move
/// when player lost. `remaining_moves` is the number of moves, that
/// player had before loosing.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LoseData {
    pub move_index: usize,
    pub remaining_moves: usize,
}
/// The state of the game.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum GameState {
    /// The game hasn't ended
    Ongoing,
    /// The game has ended with win or draw
    Ended(GameOver),
}
/// GameOver options.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum GameOver {
    /// The game has a winner
    Win(usize),
    /// The game has ended with a draw
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
