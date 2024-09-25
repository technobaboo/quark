use crate::prelude::*;
use dashmap::{mapref::one::RefMut, DashMap};
use rustc_hash::FxHasher;
use std::{hash::BuildHasherDefault, sync::atomic::AtomicU64};

pub trait Handle: Copy {
    fn from_raw(raw: u64) -> Self;
    fn into_raw(self) -> u64;
    fn validate(self) -> XrResult {
        if self.into_raw() == 0 {
            Err(XrErr::ERROR_HANDLE_INVALID)
        } else {
            Ok(())
        }
    }
}

pub trait HandleData: Sized + Send + Sync + 'static {
    type Handle: Handle;

    fn registry<'a>() -> &'a DashMap<u64, Self, BuildHasherDefault<FxHasher>>;
    fn counter<'a>() -> &'a AtomicU64;

    fn store_in_new_handle(self) -> u64 {
        let id = Self::counter()
            .fetch_update(
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
                |x| Some(x + 1),
            )
            .unwrap();
        Self::registry().insert(id, self);
        id
    }
    fn store_in_existing_handle(self, handle: Self::Handle) {
        Self::registry().insert(handle.into_raw(), self);
    }

    fn destroy(handle: Self::Handle) -> XrResult {
        handle.validate()?;
        match Self::registry().remove(&handle.into_raw()) {
            Some(_) => Ok(()),
            None => Err(XrErr::ERROR_HANDLE_INVALID),
        }
    }

    fn borrow_raw<'a>(handle: Self::Handle) -> Result<RefMut<'a, u64, Self>, XrErr> {
        handle.validate()?;
        Self::registry()
            .get_mut(&handle.into_raw())
            .ok_or(XrErr::ERROR_HANDLE_INVALID)
    }
}
