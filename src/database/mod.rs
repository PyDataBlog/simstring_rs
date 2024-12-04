mod hashdb;

pub trait SimStringDB {
    fn insert(&mut self, s: String);
    fn describe_collection(&self) -> (usize, f64, usize);
}

pub use hashdb::HashDB;
