use super::*;

#[test]
fn component_reexports_app_type() {
    let _ = std::any::type_name::<App>();
}
