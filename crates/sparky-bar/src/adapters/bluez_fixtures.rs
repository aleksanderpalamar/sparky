use zbus::fdo::ManagedObjects;
use zbus::zvariant::{OwnedValue, Value};

pub fn value(value: &str) -> OwnedValue {
    Value::from(value).try_into().unwrap()
}

pub fn insert(
    objects: &mut ManagedObjects,
    path: &str,
    interface: &str,
    properties: Vec<(&str, OwnedValue)>,
) {
    objects.insert(
        path.try_into().unwrap(),
        [(
            interface.try_into().unwrap(),
            properties
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )]
        .into(),
    );
}
