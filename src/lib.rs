pub mod unsync;
pub mod sync;

#[derive(thiserror::Error,Debug)]
pub enum PipeError{
    #[error("left is drop")]
    LeftDrop,
    #[error("right is drop")]
    RightDrop
}


