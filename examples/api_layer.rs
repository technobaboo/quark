use openxr::{
    sys::{Instance, InstanceCreateInfo, Result as XrErr},
    Entry, InstanceExtensions,
};
use quark::{api_layer, prelude::*, APILayerInstanceData};

#[handle(openxr::sys::Instance)]
pub struct InstanceData {
    instance: openxr::Instance,
}
impl APILayerInstanceData for InstanceData {
    fn create(
        entry: Entry,
        instance_info: &InstanceCreateInfo,
        instance: Instance,
    ) -> Result<Self, XrErr> {
        let app_name = unsafe {
            str_from_const_char(instance_info.application_info.application_name.as_ptr())?
        };
        println!("got your new instance chief, app's named {app_name}");
        let instance =
            unsafe { openxr::Instance::from_raw(entry, instance, InstanceExtensions::default())? };
        Ok(InstanceData { instance })
    }
    fn entry(&self) -> &Entry {
        self.instance.entry()
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
pub fn create_action_set(instance: Instance) -> XrResult {
    let data: &InstanceData = &*InstanceData::borrow_raw(instance)?;
    data.instance.create_action_set("uwu", "owo", 30)?;

    Ok(())
}
