use smithay::backend::renderer::element::Id;

pub fn anchors<'a, T>(ids: impl Iterator<Item = &'a Id>, wanted: Vec<(Id, T)>) -> Vec<(usize, T)> {
    let ids: Vec<&Id> = ids.collect();
    let mut found: Vec<(usize, T)> = wanted
        .into_iter()
        .filter_map(|(id, item)| {
            ids.iter()
                .position(|candidate| *candidate == &id)
                .map(|index| (index, item))
        })
        .collect();
    found.sort_by_key(|(index, _)| *index);
    found
}
