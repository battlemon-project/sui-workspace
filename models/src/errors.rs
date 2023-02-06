#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The Sui's event with name `{0}` is unsupported")]
    UnsupportedSuiEvent(String),
    #[error("The event must have fields")]
    EventWithoutFields,
    #[error("Failed to split event type on parts")]
    EventTypeSplit,
    #[error("The event type with name `{0}` is unsupported")]
    UnsupportedEventType(String),
    #[error("The event's field with name `{0}` doesn't exist")]
    WrongEventFieldName(String),
}
