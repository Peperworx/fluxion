use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Async trait backends
        async_trait: { feature = "async-trait" },
        // Features
        foreign: { feature = "foreign" },
        serde: { feature = "serde" },
        error_policy: { feature = "error-policy" },
    }
}