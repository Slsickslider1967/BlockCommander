use crate::Loader;

pub fn create(name: String, version: String, loader: Loader) {
    println!("Creating server '{}' running Minecraft {} with {:?} loader", name, version, loader);
}