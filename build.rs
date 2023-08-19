use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Async trait backends
        async_trait: { feature = "async-trait" },
        // Features
        foreign: { feature = "foreign" },
        serde: { feature = "serde" },
        federated: { feature = "federated" },
        notification: { feature = "notification" },
    }
}