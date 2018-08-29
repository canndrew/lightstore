// TODO: use crypto implementations with more peer-review than what dalek currently has.
// TODO: zero memory on drop for secret data (const generics would be useful for this)
// TODO: Avoid leaving secret data on the stack when constructing secret types.
// TODO: use mlock and a custom allocator to stop secret data getting swapped to disk
// TODO: get a pro to review all this.

use super::*;

mod xor_addr;
mod base32;
mod encrypt;
mod sign;
mod inspect;
mod secure;

pub use self::xor_addr::*;
pub use self::base32::*;
pub use self::encrypt::*;
pub use self::sign::*;
pub use self::inspect::*;
pub use self::secure::*;

