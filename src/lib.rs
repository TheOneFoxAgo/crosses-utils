//! A set of useful things for crosses connoisseurs. What are crosses?
//! You can find them in the russian book “Логика или фортуна”,
//! though there they are called “Война вирусов”.
//! Currently there are two useful things:
//! [`PlayerManager`] and [`IbtsBoard`].
//!
//! [`PlayerManager`]: player_manager::PlayerManager
//! [`IbtsBoard`]: ibts::IbtsBoard

#![no_std]
pub mod base;
pub mod ibts;
pub mod player_manager;
