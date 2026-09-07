// src/commands/config/mod.rs
mod serverdir;
mod port;

pub use serverdir::*;
pub use port::*;     // <-- you also never re-exported port, only serverdir