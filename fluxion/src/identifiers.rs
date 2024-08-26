//! # Identifiers
//! Fluxion needs a way to identify individual actors between systems.
//! This module provides the [`Identifier`] enum, which provides a clean method to distinguish between different actors.


/// # [`Identifier`]
/// Identifies an individual actor on a given system. There are two variants: one for actors on the current system, and one on a foreign system.
/// These are called [`Identifier::Local`] and [`Identifier::Foreign`] respectively.
pub enum Identifier<#[cfg(feature = "foreign")] 'a> {
    /// Identifies an actor on the current system. Contains the actor's id as a 64-bit integer.
    Local(u64),
    /// Identifies an actor on a given foreign system. Contains first the actor's id, then the foreign system's id as a string.
    #[cfg(feature = "foreign")]
    Foreign(u64, &'a str),
}

#[cfg(feature = "foreign")]
impl<'a> From<u64> for Identifier<'a> {
    fn from(value: u64) -> Self {
        Identifier::Local(value)
    }
}

#[cfg(not(feature = "foreign"))]
impl From<u64> for Identifier {
    fn from(value: u64) -> Self {
        Identifier::Local(value)
    }
}

/// # [`MessageID`]
/// Every foreign message is required to have a unique ID.
/// This is automatically populated by the `message` proc macro.
pub trait MessageID {
    const ID: &'static str;
}