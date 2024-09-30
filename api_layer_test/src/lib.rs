use quark::{
    openxr::{
        self,
        sys::{ActionSetCreateInfo, Instance, InstanceCreateInfo},
        Entry,
    },
    prelude::*,
    APILayerInstanceData,
};

#[quark::handle(openxr::sys::Instance)]
pub struct InstanceData {
    instance: openxr::Instance,
}
impl APILayerInstanceData for InstanceData {
    fn create(
        entry: Entry,
        instance_info: &InstanceCreateInfo,
        instance: Instance,
    ) -> XrResult<Self> {
        let app_name = instance_info
            .application_info
            .application_name
            .to_rust_string()?;
        println!("got your new instance chief, app's named {app_name}");
        let instance = unsafe {
            openxr::Instance::from_raw(entry, instance, openxr::InstanceExtensions::default())
        }?;
        Ok(InstanceData { instance })
    }
    fn entry(&self) -> &Entry {
        self.instance.entry()
    }
}

quark::api_layer! {
    instance_data: InstanceData,
    override_fns: {
        xrCreateActionSet: xr_create_action_set,
        xrDestroyActionSet: xr_destroy_action_set,
        xrCreateAction: create_action
    }
}

#[quark::handle(openxr::sys::ActionSet)]
pub struct ActionSetData {
    instance: openxr::Instance,
    action_set: openxr::ActionSet,
}

#[quark::wrap_openxr]
pub fn xr_create_action_set(
    instance: Instance,
    create_info: &ActionSetCreateInfo,
    original_action_set: &mut openxr::sys::ActionSet,
) -> XrResult {
    println!(
        "New action set named \"{}\"",
        create_info.localized_action_set_name.to_rust_string()?
    );
    let data = instance.data()?;
    let name = create_info.action_set_name.to_rust_string()?;
    let localized_name = create_info.localized_action_set_name.to_rust_string()?;

    let action_set = data
        .instance
        .create_action_set(name, localized_name, create_info.priority)?;
    *original_action_set = action_set.as_raw();

    original_action_set.add_data(ActionSetData {
        instance: data.instance.clone(),
        action_set,
    });

    Ok(())
}

#[quark::wrap_openxr]
pub fn create_action(
    action_set: openxr::sys::ActionSet,
    create_info: &openxr::sys::ActionCreateInfo,
    action: &mut openxr::sys::Action,
) -> XrResult {
    println!(
        "Created action with name \"{}\"",
        create_info.localized_action_name.to_rust_string()?
    );
    let instance = &action_set.data()?.instance;
    cvt(|| unsafe { (instance.fp().create_action)(action_set, create_info, action) })
}
