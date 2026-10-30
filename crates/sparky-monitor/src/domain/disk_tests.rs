use super::disk::{DiskCounters, Mount, owning_disk, parse_diskstats, parse_mounts, physical};

const DISKSTATS: &str = "   8       0 sda 8486 4302 938610 8480 1437 14347 1156696 50065 0 10008\n   8       1 sda1 51 0 2736 52 0 0 0 0 0 31\n 259       0 nvme0n1 423712 43105 30485973 217278 472217 590 24726603 1968586 0 79831\n 253       0 zram0 52717 0 423984 376 943553 0 8635832 19803 0 24658\n";

#[test]
fn le_somente_discos_inteiros_em_bytes() {
    let disks = vec!["nvme0n1".to_owned(), "sda".to_owned()];
    let counters = parse_diskstats(DISKSTATS, &disks);
    assert_eq!(
        counters,
        [
            DiskCounters {
                name: "sda".to_owned(),
                read: 938_610 * 512,
                written: 1_156_696 * 512
            },
            DiskCounters {
                name: "nvme0n1".to_owned(),
                read: 30_485_973 * 512,
                written: 24_726_603 * 512
            },
        ]
    );
}

#[test]
fn dispositivos_virtuais_nao_contam_como_disco() {
    assert!(physical("nvme0n1"));
    assert!(physical("sda"));
    for name in ["zram0", "loop3", "dm-0", "md127", "sr0", "ram1"] {
        assert!(!physical(name), "{name}");
    }
}

#[test]
fn montagens_de_dispositivos_reais_sem_repetir() {
    let text = "proc /proc proc rw 0 0\n/dev/nvme0n1p2 / ext4 rw 0 0\n/dev/nvme0n1p2 /var/lib/docker ext4 rw 0 0\n/dev/sda1 /mnt/meus\\040dados ext4 rw 0 0\n/dev/loop0 /snap/x squashfs ro 0 0\n";
    assert_eq!(
        parse_mounts(text),
        [
            Mount {
                device: "/dev/nvme0n1p2".to_owned(),
                path: "/".to_owned()
            },
            Mount {
                device: "/dev/sda1".to_owned(),
                path: "/mnt/meus dados".to_owned()
            },
        ]
    );
}

#[test]
fn particao_pertence_ao_disco_de_nome_mais_longo() {
    let disks = vec!["sda".to_owned(), "nvme0n1".to_owned(), "sdaa".to_owned()];
    assert_eq!(
        owning_disk("nvme0n1p3", &disks).map(String::as_str),
        Some("nvme0n1")
    );
    assert_eq!(
        owning_disk("sdaa2", &disks).map(String::as_str),
        Some("sdaa")
    );
    assert_eq!(owning_disk("sda1", &disks).map(String::as_str), Some("sda"));
    assert_eq!(owning_disk("mmcblk0p1", &disks), None);
}
