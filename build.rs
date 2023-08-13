use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Tracing
        tracing: { feature = "tracing" },
        release_tracing: { all(tracing, any(feature = "release_tracing", debug_assertions)) }
    }
}