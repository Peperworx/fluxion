//! # Identifiers
//! Fluxion needs a way to identify individual actors between systems.
//! This module provides the [`Identifier`] enum, which provides a clean method to distinguish between different actors.


/// # [`Identifier`]
/// Identifies an individual actor on a given system. There are two variants: one for actors on the current system, and one on a foreign system.
/// These are called [`Identifier::Local`] and [`Identifier::Foreign`] respectively.
pub enum Identifier<'a> {
    /// Identifies an actor on the current system. Contains the actor's id as a 64-bit integer.
    Local(u64),
    /// Identifies an actor on a given foreign system. Contains first the actor's id, then the foreign system's id as a string.
    Foreign(u64, &'a str),
}

/// # [`Identifiable`]
/// A fun trick to allow using human-readable strings as identifiers
pub enum Identifiable<'a> {
    Identifier(Identifier<'a>),
    Named(&'a str),
}