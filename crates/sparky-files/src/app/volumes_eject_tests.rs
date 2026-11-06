use super::*;

#[test]
fn ejecao_e_aceita_so_para_volume_removivel_e_serializada() {
    let mut scene = Cena::nova();
    let source = Arc::new(Source {
        calls: Mutex::default(),
        result: Ok(Vec::new()),
    });
    scene.files.services.volumes = source.clone();
    scene.files.volumes = vec![volume(None)];
    let task = volumes::eject(&mut scene.files, "/dev/sda1");
    assert_eq!(
        scene.files.volume_work,
        Some(Operation::Eject("/dev/sda1".to_owned()))
    );
    assert!(
        iced_runtime::task::into_stream(volumes::eject(&mut scene.files, "/dev/sda1")).is_none()
    );
    complete(&mut scene, task);
    assert_eq!(
        source.calls.lock().unwrap().as_slice(),
        &[Operation::Eject("/dev/sda1".to_owned())]
    );
    assert!(scene.files.volumes.is_empty());
    let mut fixed = volume(None);
    fixed.removable = false;
    scene.files.volumes = vec![fixed];
    assert!(
        iced_runtime::task::into_stream(volumes::eject(&mut scene.files, "/dev/sda1")).is_none()
    );
}
