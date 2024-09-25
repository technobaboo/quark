use openxr::{
    sys::{Instance, InstanceCreateInfo, Result as XrErr},
    Entry,
};
use quark::{api_layer, prelude::*, APILayerInstanceData};

#[handle(openxr::sys::Instance)]
pub struct InstanceData {
    entry: Entry,
}
impl APILayerInstanceData for InstanceData {
    fn create(
        entry: Entry,
        instance_info: &InstanceCreateInfo,
        _raw_instance: Instance,
    ) -> Result<Self, XrErr> {
        let app_name = unsafe {
            str_from_const_char(instance_info.application_info.application_name.as_ptr())?
        };
        println!("got your new instance chief, app's named {app_name}");
        Ok(InstanceData { entry })
    }
    fn entry(&self) -> &Entry {
        &self.entry
    }
}

api_layer! {
    instance_data: InstanceData,
    override_fns: [
        xrCreateActionSet
    ]
}

pub fn main() {}

#[openxr(xrCreateActionSet)]
pub fn create_action_set() -> XrResult {
    todo!()
}
