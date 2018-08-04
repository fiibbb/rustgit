use object;


pub trait API {
    fn put(&mut self, changes: Vec<(String, Vec<u8>)>) -> Result<Vec<(String, object::Hash)>, String>;
    fn get(&self, key: String, time: Option<u64>) -> Option<Box<object::Object>>;
}