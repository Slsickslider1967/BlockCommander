mod vanilla;
mod fabric;
mod forge;
mod neoforge;

pub use vanilla::vanilla;
pub use fabric::fabric;
pub use forge::forge;
pub use neoforge::neoforge;

use crate::Loader;
use crate::commands::config::sync;

pub fn create(name: String, version: String, loader: Loader)
{
    match loader
    {
        Loader::Vanilla => vanilla(name, version),
        Loader::Fabric => fabric(name, version),
        Loader::Forge => forge(name, version),
        Loader::NeoForge => neoforge(name, version),
    }

    sync();
}