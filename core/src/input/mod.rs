//! Input methods

pub mod telex;
pub mod vni;

pub use telex::Telex;
pub use vni::Vni;

/// Input method trait
pub trait Method {
    /// Check if key is mark key (s/f/r/x/j or 1-5)
    /// Returns mark: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    fn is_mark(&self, key: u16) -> Option<u8>;

    /// Check if key is tone key
    /// Returns: 1=hat(^) for a/e/o, 2=breve(˘) for a/o/u
    fn is_tone(&self, key: u16, prev: Option<u16>) -> Option<u8>;

    /// Check if key triggers đ
    fn is_d(&self, key: u16, prev: Option<u16>) -> bool;

    /// Check if key removes mark (z or 0)
    fn is_remove(&self, key: u16) -> bool;
}

/// Get method by id
pub fn get(id: u8) -> Box<dyn Method> {
    match id {
        1 => Box::new(Vni),
        _ => Box::new(Telex),
    }
}
