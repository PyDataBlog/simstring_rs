mod hashdb;

pub trait SimStringDB {
    fn insert(&mut self, s: String);
    fn describe_collection(&self) -> (usize, f64, usize);
    fn get_max_feature_size(&self) -> usize;
}

pub use hashdb::HashDB;
