#[derive(thiserror::Error, Debug)]
pub enum ConversionError {
    #[error(transparent)]
    PostcardErr(#[from] postcard::Error),
    #[error("This message type cannot actually be sent")]
    NonConvertable,
}