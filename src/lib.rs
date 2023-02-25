pub mod sync;
pub mod unsync;

#[derive(thiserror::Error, Debug)]
pub enum PipeError {
    #[error("left is drop")]
    LeftDrop,
    #[error("right is drop")]
    RightDrop,
}
