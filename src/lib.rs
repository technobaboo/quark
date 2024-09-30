pub mod handle;
pub mod util;
pub mod prelude {
    pub use openxr::sys::Result as XrErr;
    pub use proc_macros::*;
    pub type XrResult<T> = Result<T, XrErr>;
    pub use crate::handle::*;
    pub use crate::util::*;
    pub use proc_macros::handle;
}

pub use openxr;
use openxr::{sys::InstanceCreateInfo, Entry};
use prelude::*;
pub use proc_macros::*;

pub trait APILayerInstanceData: Sized + Send + Sync + 'static {
    fn create(
        entry: Entry,
        instance_info: &InstanceCreateInfo,
        raw_instance: openxr::sys::Instance,
    ) -> XrResult<Self>;
    fn entry(&self) -> &Entry;
}

#[macro_export]
macro_rules! api_layer {
	(
		instance_data: $instance_data:ty,
		override_fns: {
			$($fn_name:ident: $override_fn:ident),*
		}
	) => {
		/// # Safety
		/// don't be stupid
		#[allow(non_snake_case)]
		#[$crate::export_openxr(xrNegotiateLoaderApiLayerInterface)]
		pub unsafe fn negotiate_loader_api_layer_interface(
			loader_info: &openxr::sys::loader::XrNegotiateLoaderInfo,
			_api_layer_name: *const u8,
			api_layer_request: &mut openxr::sys::loader::XrNegotiateApiLayerRequest,
		) -> $crate::prelude::XrResult<()> {
			use openxr::sys::loader::*;
			use openxr::sys::CURRENT_API_VERSION;
			if loader_info.ty != XrNegotiateLoaderInfo::TYPE
				|| loader_info.struct_version != XrNegotiateLoaderInfo::VERSION
				|| loader_info.struct_size != size_of::<XrNegotiateLoaderInfo>()
			{
				return Err(XrErr::ERROR_INITIALIZATION_FAILED);
			}
			if api_layer_request.ty != XrNegotiateApiLayerRequest::TYPE
				|| api_layer_request.struct_version != XrNegotiateApiLayerRequest::VERSION
				|| api_layer_request.struct_size != size_of::<XrNegotiateApiLayerRequest>()
			{
				return Err(XrErr::ERROR_INITIALIZATION_FAILED);
			}
			if CURRENT_API_VERSION > loader_info.max_api_version
				|| CURRENT_API_VERSION < loader_info.min_api_version
			{
				eprintln!(
								"OpenXR API Layer doesn't support major version {} < {} < {}",
								loader_info.max_api_version, CURRENT_API_VERSION, loader_info.min_api_version
				);
				return Err(XrErr::ERROR_INITIALIZATION_FAILED);
			}
			api_layer_request.layer_interface_version = CURRENT_LOADER_API_LAYER_VERSION;
			api_layer_request.layer_api_version = CURRENT_API_VERSION;
			api_layer_request.get_instance_proc_addr = Some(xr_get_instance_proc_addr);
			api_layer_request.create_api_layer_instance = Some(xr_create_api_layer_instance);
			Ok(())
		}

		/// # Docs
		/// [xrGetInstanceProcAddr](https://www.khronos.org/registry/OpenXR/specs/1.0/html/xrspec.html#xrGetInstanceProcAddr)
		/// # Safety
		/// you are gay
		#[allow(non_snake_case, unreachable_code)]
		pub unsafe extern "system" fn xr_get_instance_proc_addr(
			instance: openxr::sys::Instance,
			name: *const i8,
			function: *mut Option<openxr::sys::pfn::VoidFunction>,
		) -> openxr::sys::Result {
			let instance_data = match instance.data() {
				Ok(instance_data) => instance_data,
				Err(e) => return e,
			};
			let Ok(rusty_name) = <*const i8 as $crate::util::Rustify>::to_rust_string(&name) else {
				return openxr::sys::Result::ERROR_VALIDATION_FAILURE;
			};
			match rusty_name {
				"xrDestroyInstance" => {
					*function = Some(std::mem::transmute::<*const (), unsafe extern "system" fn()>(
						xr_destroy_instance as *const (),
					));
					openxr::sys::Result::SUCCESS
				},
				$(stringify!($fn_name) => {
					*function = Some(std::mem::transmute::<*const (), unsafe extern "system" fn()>(
						$override_fn as *const (),
					));
					openxr::sys::Result::SUCCESS
				},)*
				_ => (instance_data.entry().fp().get_instance_proc_addr)(instance, name, function),
			}
		}

		/// # Safety
		/// you are gay
		#[allow(non_snake_case)]
		#[$crate::wrap_openxr]
		pub unsafe fn xr_create_api_layer_instance(
			info: *const openxr::sys::InstanceCreateInfo,
			api_layer_info: *const openxr::sys::loader::ApiLayerCreateInfo,
			instance: *mut openxr::sys::Instance,
		) -> XrResult {
			if info.is_null() || api_layer_info.is_null() || instance.is_null() {
				return Err(openxr::sys::Result::ERROR_VALIDATION_FAILURE);
			}
			let instance_info = &*info;
			let layer_info = &*api_layer_info;
			((*layer_info.next_info).next_create_api_layer_instance)(
				info,
				api_layer_info,
				instance,
			);
			let entry = openxr::Entry::from_get_instance_proc_addr(
				(*layer_info.next_info).next_get_instance_proc_addr,
			)?;
			let instance_data =
				<$instance_data as APILayerInstanceData>::create(entry, instance_info, *instance)?;
			(*instance).add_data(instance_data);
			Ok(())
		}
	};
}
