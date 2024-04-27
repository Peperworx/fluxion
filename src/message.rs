use slacktor::Message;

/// # [`IndeterminateMessage`]
/// An indeterminate message is a message for which it has not yet been determined whether it will be serialized.
/// Because of this, indeterminate messages require serde traits to be implemented, which is not the case with local messages.
#[cfg(feature = "serde")]
pub trait IndeterminateMessage: Message + serde::Serialize + for<'a> serde::Deserialize<'a> {}

#[cfg(feature = "serde")]
impl<T> IndeterminateMessage for T
where T: Message + serde::Serialize + for<'a> serde::Deserialize<'a> {}


/// # [`IndeterminateMessage`]
/// An indeterminate message is a message for which it has not yet been determined whether it will be serialized.
/// Because of this, indeterminate messages require serde traits to be implemented, which is not the case with local messages.
#[cfg(not(feature = "serde"))]
pub trait IndeterminateMessage: Message {}

#[cfg(not(feature = "serde"))]
impl<T: Message> IndeterminateMessage for T {}
