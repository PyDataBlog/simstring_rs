mod hashdb;

pub trait SimStringDB {
    fn describe_collection(&self) -> (usize, f64, usize);
}
