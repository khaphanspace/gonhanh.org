//! Static lookup tables for matrix dispatch
//!
//! All tables are const and zero-initialized at compile time.
//! Total memory: ~1.4KB (without Bloom filter)
//!
//! ## Memory Budget
//!
//! | Category | Size | Purpose |
//! |----------|------|---------|
//! | LETTER_CLASS | 26 bytes | Vowel/consonant classification |
//! | KEY_CAT_TELEX | 38 bytes | Key to category mapping |
//! | DISPATCH | 40 bytes | State x Category -> Action |
//! | DEFER | 8 bytes | Pending resolution |
//! | REVERT_KEY | 11 bytes | Transform revert triggers |
//! | VN Validation | ~700 bytes | Phonotactic rules |
//! | EN Detection | ~400 bytes | Pattern detection |
//! | Placement | ~250 bytes | Tone/modifier placement |

pub mod letter_class;
pub mod key_category;
pub mod dispatch;
pub mod defer;
pub mod revert;
pub mod vietnamese;
pub mod english;
pub mod placement;

// Re-exports
pub use letter_class::{LetterClass, LETTER_CLASS};
pub use key_category::{KeyCategory, KEY_CAT_TELEX};
pub use dispatch::{Action, DISPATCH};
pub use defer::{DeferType, DEFER};
pub use revert::{REVERT_KEY, should_revert};
