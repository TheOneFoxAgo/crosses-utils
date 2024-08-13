//! This module defines [`PlayerManager`] structure, which is designed to help with player state management.
//! The main methods are [`advance`] and [`reverse`]. [`advance`] changes the state as it is expected after the player performs a move.
//! It updates the counters, changes the player if necessary, and so on, the [`reverse`] method does the same thing,
//! but in reverse as the name suggests. Since this structure deals only with player state and does not interact with the field,
//! [`PlayerManager`] and [`BoardManager`] can become out of sync if used carelessly:
//!
//! ```
//! fn bad(
//!     pm: &mut PlayerManager<Vec<Option<LoseData>>>,
//!     bm: &mut impl BoardManager,
//!     x: usize,
//!     y: usize,
//! ) -> Result<(), Box<dyn Error>> {
//!     let player = pm.current_player();
//!     let f = |p| false; // Placeholder closure. Completely meaningless.
//!     pm.advance(f, f); // state has changed
//!     bm.make_move(x, y, player)?; // if returns Err, move wasn't done and we end up in incorrect state
//!     Ok(()) // Nothing is ok!!!! It's totally wrong!!!
//! }
//! ```
//!
//! The correct usage of this struct looks something like this:
//! ```
//! fn good(
//!     pm: &mut PlayerManager<Vec<Option<LoseData>>>,
//!     bm: &mut impl BoardManager,
//!     x: usize,
//!     y: usize,
//! ) -> Result<(), Box<dyn Error>> {
//!     // first, we check if game has ended
//!     if self.game_state() != GameState::Ongoing {
//!         return Err("The game has already ended!!!")    
//!     }
//!     // Then we try to make move
//!     bm.make_move(x, y, pm.current_player())?;
//!     let f = |p| false; // Placeholder closure. Completely meaningless.
//!     // And in the end we advance our state
//!     pm.advance(f,f);
//!     Ok(())
//! }
//! ```
//! [`advance`]: PlayerManager::advance
//! [`reverse`]: PlayerManager::reverse
//! [`BoardManager`]: crate::board_manager::BoardManager

use core::{fmt::Display, ops::IndexMut};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Helper structure to track players' state during game.
/// `S` - is type of storage. It can be Vec or simple array.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PlayerManager<S: IndexMut<usize, Output = Option<LoseData>>>
{
    remaining_moves: usize,
    max_moves: usize,
    current_player: usize,
    max_players: usize,
    current_move: usize,
    game_state: GameState,
    losers: S,
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
    /// let pm = PlayerManager::new(4, 4, [None;4]);
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
    ///
    /// Attention! This function must be called only after checking game
    /// state and trying to perform move.
    /// Calling this before, can lead to incorrect state.
    /// See [module level documentation](self) for more examples.
    /// # Example
    /// ```
    /// fn make_move(
    ///     pm: &mut PlayerManager<Vec<Option<LoseData>>,
    ///     bm: &mut impl BoardManager,
    ///     x: usize,
    ///     y: usize,
    ///     log: &mut Vec<(usize, uszie)>,
    /// ) -> Result<(), Box<dyn Error>> {
    ///     if self.game_state() != GameState::Ongoing {
    ///         return Err("The game has already ended!!!")    
    ///     }
    ///
    ///     bm.make_move(x, y, pm.current_player())?;
    ///     let f = |p| false; // Placeholder closure. Completely meaningless.
    ///     log.push((x, y));
    ///     pm.advance(f, f);
    ///     Ok(())
    /// }
    /// ```
    pub fn advance(
        &mut self,
        is_ran_out_of_moves: impl Fn(usize) -> bool,
        is_ran_out_of_crosses: impl Fn(usize) -> bool,
    ) {
        if self.game_state() != GameState::Ongoing {
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
    ///
    /// Attention! This function must be called only after checking game
    /// state and trying to cancell move.
    /// Calling this before, can lead to incorrect state.
    /// # Example
    /// ```
    /// fn cancel_move(
    ///     pm: &mut PlayerManager<Vec<Option<LoseData>>,
    ///     bm: &mut impl BoardManager,
    ///     log: &mut Vec<(usize, uszie)>,
    /// ) -> Result<(), Box<dyn Error>> {
    ///     if pm.current_move() == 0 {
    ///         return Err("The game hasn't started yet!!!")
    ///     }
    ///     let (x, y) = log[pm.current_move];
    ///     let player = // some fancy way of determining player
    ///     bm.cancel_move()?;
    ///     pm.reverse(player);
    ///     log.pop();
    ///     Ok(())
    /// }
    /// ```
    pub fn reverse(&mut self, player: usize) {
        self.current_move -= 1;
        if let Some(LoseData {
            move_index: _,
            remaining_moves,
        }) = self.losers[player]
        {
            self.remaining_moves = remaining_moves;
            let mut loser_idx = player;
            loop {
                if matches!(
                    self.losers[loser_idx],
                    Some(LoseData {
                        move_index, 
                        remaining_moves: _
                    }) if move_index == self.current_move )
                {
                    self.losers[loser_idx] = None
                }
                if loser_idx == self.current_player {
                    break;
                } else {
                    loser_idx = (loser_idx + 1) % self.max_players;
                }
            }
        } else {
            self.remaining_moves = if self.remaining_moves == 0 
            {self.max_moves} else {self.remaining_moves - 1};
        }
        self.current_player = player;
    }
    /// Returns the remaining number of current player's moves
    /// # Example
    /// ```
    /// let pm = PlayerManager::new(4, 4, [None;4]);
    /// assert_eq!(pm.remaining_moves(), 4);
    /// pm.advance(|_| false, |_| false);
    /// assert_eq!(pm.remaining_moves(), 3);
    /// ```
    pub fn remaining_moves(&self) -> usize {
        self.remaining_moves
    }
    /// Returns max number of moves for the game
    /// # Example
    /// ```
    /// let pm = PlayerManager::new(4, 4, [None;4]);
    /// assert_eq!(pm.max_moves(), 4);
    /// ```
    pub fn max_moves(&self) -> usize {
        self.max_moves
    }
    /// Returns the index of current player
    /// # Example
    /// ```
    /// let pm = PlayerManager::new(4, 4, [None;4]);
    /// assert_eq!(pm.current_player(), 0);
    /// ```
    pub fn current_player(&self) -> usize {
        self.current_player
    }
    /// Return the initial number of players
    /// # Example
    /// ```
    /// let pm = PlayerManager::new(4, 3, [None;4]);
    /// assert_eq!(pm.max_players(), 3);
    /// ```
    pub fn max_players(&self) -> usize {
        self.max_players
    }
    /// Returns info about each player success.
    /// `None` - player hasn't lost yet
    /// `Some(lose_data)` - player has lost :(
    /// # Example
    /// ```
    /// let pm = PlayerManager::new(4, 4, [None;4]);
    /// pm.advance(|_| true, |_| true); // this will make all players loose. bad for them
    /// assert_ne!(pm.losers[0], None);
    /// ```
    pub fn losers(&self) -> &S {
        &self.losers
    }
    /// Return the current move index
    /// # Example
    /// ```
    /// let pm = PlayerManager::new(4, 4, [None;4]);
    /// assert_eq!(pm.current_move(), 0);
    /// pm.advance(|_| false, |_| false);
    /// assert_eq!(pm.current_move(), 1);
    /// ```
    pub fn current_move(&self) -> usize {
        self.current_move
    }
    /// Returns state of the game. If it isn't `GameState::Ongoing`,
    /// the manager will refuse to advance.
    /// # Example
    /// ```
    /// let pm = PlayerManager::new(4, 4, [None;4]);
    /// assert_eq!(pm.game_state(), GameState::Ongoing);
    /// ```
    pub fn game_state(&self) -> GameState {
        self.game_state
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
                    if is_ran_out_of_crosses(not_loser_idx)
                    {
                        self.losers[not_loser_idx] = Some(LoseData {
                            move_index: self.current_move,
                            remaining_moves: 0,
                        });
                        maybe_not_losers -= 1;
                    } else if is_ran_out_of_moves(not_loser_idx)
                    {
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
/// The state of the game. Either 'Ongoing' or 'Ended'
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum GameState {
    Ongoing,
    Ended(GameOver)
}
/// GameOver options. Either `Win` or `Draw`
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
