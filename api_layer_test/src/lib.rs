use openxr::{
    sys::{Instance, InstanceCreateInfo, Result as XrErr},
    Entry,
};
use quark::{openxr, prelude::*, APILayerInstanceData};

#[handle(quark::openxr::sys::Instance)]
pub struct InstanceData {
    entry: Entry,
    instance: openxr::sys::Instance,
    instance_functions: openxr::raw::Instance,
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
        let instance_functions = unsafe { openxr::raw::Instance::load(&entry, instance) }?;
        Ok(InstanceData {
            entry,
            instance,
            instance_functions,
        })
    }
    fn entry(&self) -> &Entry {
        &self.entry
    }
}

quark::api_layer! {
    instance_data: InstanceData,
    override_fns: {
        xrCreateActionSet: create_action_set
    }
}

#[quark::wrap_openxr]
pub fn create_action_set(
    instance: Instance,
    create_info: &openxr::sys::ActionSetCreateInfo,
    action_set: &mut openxr::sys::ActionSet,
) -> XrResult {
    println!(
        "new action set named {}",
        create_info.action_set_name.to_rust_string()
    );
    let data = InstanceData::borrow_raw(instance).unwrap();
    cvt(|| unsafe {
        (data.instance_functions.create_action_set)(instance, create_info, action_set)
    })
}
