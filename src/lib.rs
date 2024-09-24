mod prelude {
    pub use openxr_sys::Result as XrErr;
    pub use proc_macros::*;
    pub type XrResult = Result<(), XrErr>;
    pub use crate::handle::Handle;
    pub use crate::util::*;
}

mod handle;
mod util;
