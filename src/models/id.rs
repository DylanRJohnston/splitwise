use std::hash::Hash;

pub trait ID {
    type ID: Hash + Eq + Send + Sync;

    fn id(&self) -> Self::ID;
}
