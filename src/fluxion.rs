use alloc::sync::Arc;
use maitake_sync::RwLock;
use slacktor::Slacktor;





/// # [`Fluxion`]
/// Contains the core actor management functionality of fluxion
pub struct Fluxion {
    /// The underlying slacktor instance.
    /// This is wrapped in an [`Arc`]` and [`RwLock`]` to allow concurrent access from different tasks.
    /// The [`RwLock`] is used instead of a mutex because it can be assumed that actor references
    /// will be retrieved more often than actors are created.
    slacktor: Arc<RwLock<Slacktor>>,
    /// The identifier of this system as a string
    system_id: Arc<str>,
}

impl Fluxion {
    /// # [`new`]
    /// Creates a new [`Fluxion`] instance with the given system id
    #[must_use]
    pub fn new(id: &str) -> Self {
        Self {
            slacktor: Arc::new(RwLock::new(Slacktor::new())),
            system_id: id.into(),
        }
    }

    /// # [`get_id`]
    /// Gets the system's id
    #[must_use]
    pub fn get_id(&self) -> &str {
        &self.system_id
    }

    /// # [`add`]
    /// Adds an actor to the local instance.
    /// <div class = "info">
    /// Locks the underlying RwLock as write.
    /// </div>
    pub async fn add<A>(&self, actor: A) {
        todo!()
    }

}