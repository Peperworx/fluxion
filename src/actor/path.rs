//! Contains a structure to represent an actor's path

/// # ActorPath
/// Provides a wrapper for working with actor paths
#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Eq)]
pub struct ActorPath {
    /// The ID of the contained actor
    actor: String,
    /// The systems in the path
    systems: Vec<String>,
}

impl ActorPath {
    /// Creates a new empty path from a schema string
    /// Returns None if the schema is empty
    pub fn new(schema: &str) -> Option<Self> {
        // If the string is empty, return None
        if schema.is_empty() {
            return None;
        }

        // Split into parts
        let parts = schema.split(":").map(|v| v.to_string()).collect::<Vec<_>>();

        // Get the first as the actor id
        let actor = parts.last()?.clone();

        // Get the rest as the systems
        let systems = parts[0..parts.len() - 1].to_vec();

        Some(ActorPath { actor, systems })
    }

    /// Gets the individual components of the path
    pub fn systems(&self) -> Vec<String> {
        self.systems.clone()
    }

    /// Gets the actor name
    pub fn actor(&self) -> &str {
        &self.actor
    }

    /// Gets the first system on the actor's path, returning none if there are  no systems
    pub fn first(&self) -> Option<&str> {
        self.systems.get(0).map(|v| v.as_str())
    }

    /// Pops off the first system, returning the remainder
    pub fn popfirst(&self) -> ActorPath {
        // If there are no systems, then return self
        if self.systems.len() == 0 {
            return self.clone();
        }

        // Get the systems after the first one
        let systems = self.systems[1..].to_vec();

        // Create a new actor path and return
        ActorPath {
            actor: self.actor.clone(),
            systems,
        }
    }
}


impl ToString for ActorPath {
    fn to_string(&self) -> String {
        self.systems.join(":") + ":" + &self.actor
    }
}