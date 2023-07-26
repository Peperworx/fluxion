//! Contains a structure to represent an actor's path

/// # ActorPath
/// [`ActorPath`] provides a wrapper for working with actor paths
#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Eq)]
pub struct ActorPath {
    /// The ID of the actor represented by the path
    actor: String,
    /// The systems in the path
    systems: Vec<String>,
}

impl ActorPath {
    /// Creates a new path from a schema &str, delimited by ':', where the last item is the actor's id and the others are the path of systems through which the actor can be accessed.
    /// Returns None if the schema is empty
    pub fn new(schema: &str) -> Option<Self> {
        // If the string is empty, return None
        if schema.is_empty() {
            return None;
        }

        // Split into parts
        let parts = schema.split(':').map(|v| v.to_string()).collect::<Vec<_>>();

        // Get the first as the actor id
        let actor = parts.last()?.clone();

        // Get the rest as the systems
        let systems = parts[0..parts.len() - 1].to_vec();

        Some(ActorPath { actor, systems })
    }

    /// Gets the individual systems in the path
    pub fn systems(&self) -> &[String] {
        &self.systems
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
        if self.systems.is_empty() {
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



impl std::fmt::Display for ActorPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(self.systems.join(":") + ":" + &self.actor))
    }
}