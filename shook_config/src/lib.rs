mod secret;
pub use secret::Secret;

mod ephemeral;
pub use ephemeral::Ephemeral;

fn redact(s: &str) -> impl std::fmt::Debug {
    struct NoDebug(String);
    impl std::fmt::Debug for NoDebug {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&*self.0)
        }
    }

    NoDebug(format!("{{len = {}}}", s.len()))
}
