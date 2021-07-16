#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate libmww;

mod error;
mod project;

pub use error::*;
pub use project::*;

use libmww::database::*;
