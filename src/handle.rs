use crate::prelude::*;
use std::{hash::BuildHasherDefault, sync::atomic::AtomicU64};

use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use once_cell::sync::Lazy;
use rustc_hash::FxHasher;

pub trait Handle: Copy {
    type HandleType;

    fn raw(self) -> u64;

    fn from(raw: u64) -> Self;

    fn registry<'a>() -> &'a Lazy<DashMap<u64, Self::HandleType, BuildHasherDefault<FxHasher>>>;

    fn counter<'a>() -> &'a AtomicU64;

    fn create(item: Self::HandleType) -> Self {
        let id = Self::counter()
            .fetch_update(
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
                |x| Some(x + 1),
            )
            .unwrap();
        Self::registry().insert(id, item);
        Self::from(id)
    }

    fn destroy(self) -> XrResult {
        if self.raw() == 0 {
            Err(XrErr::ERROR_HANDLE_INVALID)
        } else {
            match Self::registry().remove(&self.raw()) {
                Some(_) => Ok(()),
                None => Err(XrErr::ERROR_HANDLE_INVALID),
            }
        }
    }

    fn get_mut<'a>(
        self,
    ) -> Result<RefMut<'a, u64, Self::HandleType, BuildHasherDefault<FxHasher>>, XrErr> {
        if self.raw() == 0 {
            Err(XrErr::ERROR_HANDLE_INVALID)
        } else {
            Self::registry()
                .get_mut(&self.raw())
                .ok_or(XrErr::ERROR_HANDLE_INVALID)
        }
    }

    fn get<'a>(
        self,
    ) -> Result<Ref<'a, u64, Self::HandleType, BuildHasherDefault<FxHasher>>, XrErr> {
        if self.raw() == 0 {
            Err(XrErr::ERROR_HANDLE_INVALID)
        } else {
            Self::registry()
                .get(&self.raw())
                .ok_or(XrErr::ERROR_HANDLE_INVALID)
        }
    }

    fn is_null(self) -> bool {
        if self.raw() == 0 {
            true
        } else {
            Self::registry().get(&self.raw()).is_none()
        }
    }
}
