use object;


pub trait API {
    fn put(&mut self, key: String, value: Vec<u8>) -> Result<object::Hash, String>;
    fn get(&self, key: String, time: Option<u64>) -> Option<Box<object::Object>>;
}