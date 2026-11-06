pub(crate) fn device_name(vendor: Option<&str>, model: Option<&str>) -> Option<String> {
    let vendor = vendor.map(str::trim).filter(|value| !value.is_empty());
    let model = model.map(str::trim).filter(|value| !value.is_empty());
    match (vendor, model) {
        (Some(vendor), Some(model)) if model.to_lowercase().starts_with(&vendor.to_lowercase()) => {
            Some(model.to_owned())
        }
        (Some(vendor), Some(model)) => Some(format!("{vendor} {model}")),
        (Some(vendor), None) => Some(vendor.to_owned()),
        (None, Some(model)) => Some(model.to_owned()),
        (None, None) => None,
    }
}
