use crate::prelude::*;
use dashmap::{mapref::one::MappedRefMut, DashMap};
use std::{
    any::{Any, TypeId},
    sync::LazyLock,
};

static DATA_REGISTRY: LazyLock<DashMap<(u64, TypeId), Box<dyn Any + Send + Sync + 'static>>> =
    LazyLock::new(DashMap::default);

type DataResult<'a, Data> = std::result::Result<
    MappedRefMut<'a, (u64, TypeId), Box<(dyn Any + Send + Sync + 'static)>, Data>,
    openxr::sys::Result,
>;
pub trait Handle<Data: Any + Send + Sync + 'static>: Any + Copy {
    fn from_raw(raw: u64) -> Self;
    fn into_raw(self) -> u64;

    fn data(&self) -> DataResult<'_, Data> {
        DATA_REGISTRY
            .get_mut(&(self.into_raw(), TypeId::of::<Self>()))
            .ok_or(XrErr::ERROR_HANDLE_INVALID)?
            .try_map(|d| d.downcast_mut::<Data>())
            .map_err(|_| XrErr::ERROR_HANDLE_INVALID)
    }
    fn validate(self) -> XrResult {
        if self.into_raw() == 0 {
            Err(XrErr::ERROR_HANDLE_INVALID)
        } else {
            Ok(())
        }
    }
    fn add_data(self, data: Data) {
        DATA_REGISTRY.insert((self.into_raw(), TypeId::of::<Self>()), Box::new(data));
    }
    fn remove_data(self) {
        DATA_REGISTRY.remove(&(self.into_raw(), TypeId::of::<Self>()));
    }
}

// pub trait HandleData: Sized + Send + Sync + 'static {
//     fn registry<'a>() -> &'a DashMap<u64, Self, BuildHasherDefault<FxHasher>>;

//     fn store_in_new_handle(self) -> u64 {

//         let id = Self::counter()
//             .fetch_update(
//                 std::sync::atomic::Ordering::SeqCst,
//                 std::sync::atomic::Ordering::SeqCst,
//                 |x| Some(x + 1),
//             )
//             .unwrap();
//         Self::registry().insert(id, self);
//         id
//     }
//     fn store_in_existing_handle(self, handle: u64) {
//         Self::registry().insert(handle, self);
//     }

//     fn destroy(handle: u64) -> XrResult {
//         match Self::registry().remove(&handle) {
//             Some(_) => Ok(()),
//             None => Err(XrErr::ERROR_HANDLE_INVALID),
//         }
//     }

//     fn borrow_raw<'a>(handle: u64) -> Result<RefMut<'a, u64, Self>, XrErr> {
//         Self::registry()
//             .get_mut(&handle)
//             .ok_or(XrErr::ERROR_HANDLE_INVALID)
//     }
// }
