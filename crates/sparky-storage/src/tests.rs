use crate::{Operation, Volume, external_volumes, parse};

const DISKS: &str = r#"{"blockdevices":[
 {"name":"nvme0n1","path":"/dev/nvme0n1","type":"disk","children":[
  {"name":"nvme0n1p1","path":"/dev/nvme0n1p1","type":"part","fstype":"vfat","mountpoint":"/boot"},
  {"name":"nvme0n1p2","path":"/dev/nvme0n1p2","type":"part","fstype":"ext4","mountpoint":"/"},
  {"name":"nvme0n1p3","path":"/dev/nvme0n1p3","type":"part","fstype":"ntfs","size":"320G","label":"DADOS"}]},
 {"name":"sda","path":"/dev/sda","type":"disk","rm":true,"children":[
  {"name":"sda1","path":"/dev/sda1","type":"part","fstype":"vfat","size":"32G","label":"USB"}]},
 {"name":"sdb1","path":"/dev/sdb1","type":"part","fstype":"ext4","mountpoint":"/mnt/backup"},
 {"name":"sdc1","path":"/dev/sdc1","type":"part","fstype":"ext4","mountpoint":"/run/media/p/SSD"}
]}"#;

#[test]
fn lista_ssd_interno_desmontado_usb_e_montagens_personalizadas() {
    let volumes = parse(DISKS).unwrap_or_default();
    let devices: Vec<_> = volumes.iter().map(|volume| volume.path.as_str()).collect();
    assert_eq!(devices, ["/dev/nvme0n1p3", "/dev/sda1", "/dev/sdb1", "/dev/sdc1"]);
    assert!(!volumes[0].mounted());
    assert!(volumes[1].removable);
    assert_eq!(volumes[2].mountpoint.as_deref(), Some("/mnt/backup"));
}

#[test]
fn painel_preserva_o_filtro_de_volumes_externos() {
    let volumes = external_volumes(DISKS).unwrap_or_default();
    assert_eq!(
        volumes.iter().map(|volume| volume.path.as_str()).collect::<Vec<_>>(),
        ["/dev/sda1", "/dev/sdc1"]
    );
}

#[test]
fn painel_mantem_os_tipos_e_montagens_que_ja_oferecia() {
    let output = r#"{"blockdevices":[{"path":"/dev/sda1","type":"part","fstype":"ext4","rm":true,"mountpoint":"/var/backups/USB"},{"path":"/dev/dm-1","type":"crypt","fstype":"ext4","rm":true}]}"#;
    let volumes = external_volumes(output).unwrap_or_default();
    assert_eq!(
        volumes.iter().map(|volume| volume.path.as_str()).collect::<Vec<_>>(),
        ["/dev/sda1"]
    );
}

#[test]
fn nao_oferece_sistema_swap_containers_criptografados_ou_caminhos_invalidos() {
    let output = r#"{"blockdevices":[
     {"path":"/dev/a","type":"part","fstype":"ext4","mountpoints":["/dados","/var/lib"]},
     {"path":"/dev/b","type":"part","fstype":"swap"},
     {"path":"/dev/c","type":"part","fstype":"crypto_LUKS"},
     {"path":"/dev/../etc/passwd","type":"part","fstype":"ext4"},
     {"path":"--help","type":"part","fstype":"ext4"},
     {"path":"/dev/loop0","type":"loop","fstype":"ext4"}
    ]}"#;
    assert!(parse(output).unwrap_or_default().is_empty());
    assert!(parse("invalid").is_err());
}

#[test]
fn encontra_sistemas_de_arquivos_em_lvm_e_criptografia_desbloqueada() {
    let output = r#"{"blockdevices":[{"path":"/dev/dm-1","type":"crypt","fstype":"ext4","mountpoints":[null,"/mnt/cofre"]},{"path":"/dev/mapper/dados","type":"lvm","fstype":"ext4"}]}"#;
    let volumes = parse(output).unwrap_or_default();
    assert_eq!(volumes.len(), 2);
    assert_eq!(volumes[0].mountpoint.as_deref(), Some("/mnt/cofre"));
}

#[test]
fn particao_de_dados_montada_dentro_da_pasta_do_usuario_permanece_visivel() {
    let output = r#"{"blockdevices":[{"path":"/dev/sdb1","type":"part","fstype":"ext4","mountpoint":"/home/ana/Games"},{"path":"/dev/sdc1","type":"part","fstype":"ext4","mountpoint":"/home"}]}"#;
    let volumes = parse(output).unwrap_or_default();
    assert_eq!(volumes.len(), 1);
    assert_eq!(volumes[0].path, "/dev/sdb1");
}

#[test]
fn operacoes_usam_o_caminho_do_dispositivo_como_argumento_separado() {
    let mount = Operation::Mount("/dev/sda1".to_owned());
    assert_eq!(mount.command(), Some(("mount", "/dev/sda1")));
    assert_eq!(
        Operation::Unmount("/dev/sda1".to_owned()).command(),
        Some(("unmount", "/dev/sda1"))
    );
    assert_eq!(Operation::List.command(), None);
    let volume = Volume {
        path: "/dev/sda1".to_owned(),
        label: String::new(),
        size: "32G".to_owned(),
        filesystem: "vfat".to_owned(),
        mountpoint: None,
        removable: true,
    };
    assert_eq!(volume.name(), "/dev/sda1");
}
