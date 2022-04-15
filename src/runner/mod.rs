mod actions;
mod runner;
mod util;

pub use actions::Action;
pub use runner::Runner;
use util::{should_merge, should_pop, should_update};
