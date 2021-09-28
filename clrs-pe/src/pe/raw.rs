mod indices;
mod signatures;
mod tables;

pub use self::indices::*;
pub use self::signatures::*;
pub use self::tables::*;

#[derive(Clone, Copy, Debug)]
pub struct PeCtx {
    // TODO: fill dynamic infomation
}
