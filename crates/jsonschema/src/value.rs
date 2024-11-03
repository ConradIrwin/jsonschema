pub trait Value: std::fmt::Debug + Clone + Sized {
    fn a(&self) -> bool;
}

impl Value for serde_json::Value {
    fn a(&self) -> bool {
        self
    }
}
