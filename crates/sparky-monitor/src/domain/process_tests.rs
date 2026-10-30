use super::process::{
    ProcStat, display_name, io_bytes, kernel_thread, parse_stat, program, real_uid, resident_pages,
};

#[test]
fn le_stat_mesmo_com_parenteses_e_espacos_no_nome() {
    let text =
        "4242 (Web (Content)) S 4200 4200 4200 0 -1 4194304 127 0 1 0 150 50 0 0 25 5 1 0 3731314";
    assert_eq!(
        parse_stat(text),
        Some(ProcStat {
            pid: 4242,
            comm: "Web (Content)".to_owned(),
            ppid: 4200,
            ticks: 200,
            started: 3_731_314
        })
    );
    assert_eq!(parse_stat("lixo"), None);
}

#[test]
fn threads_do_kernel_sao_reconhecidas() {
    let kthread = parse_stat("33 (kworker/0:1) I 2 0 0 0 -1 0 0 0 0 0 0 0 0 0").unwrap();
    assert!(kernel_thread(&kthread));
    let user = parse_stat("900 (zsh) S 1 0 0 0 -1 0 0 0 0 0 0 0 0 0").unwrap();
    assert!(!kernel_thread(&user));
}

#[test]
fn memoria_io_e_dono_do_processo() {
    assert_eq!(resident_pages("1578 576 549 7 0 151 0"), Some(576));
    let io =
        "rchar: 2884\nwchar: 8\nread_bytes: 4096\nwrite_bytes: 8192\ncancelled_write_bytes: 0\n";
    assert_eq!(io_bytes(io), Some((4096, 8192)));
    assert_eq!(io_bytes("rchar: 1\n"), None);
    assert_eq!(
        real_uid("Name:\tzsh\nUid:\t1000\t1000\t1000\t1000\n"),
        Some(1000)
    );
}

#[test]
fn nome_completo_desfaz_o_corte_de_quinze_letras_do_kernel() {
    let cmdline = "/usr/bin/sparky-notifications\0--flag\0";
    assert_eq!(
        display_name("sparky-notifica", cmdline),
        "sparky-notifications"
    );
    assert_eq!(
        display_name("Web Content", "/usr/lib/firefox/firefox\0-contentproc\0"),
        "Web Content"
    );
    assert_eq!(
        program("Web Content", "/usr/lib/firefox/firefox\0-contentproc\0"),
        "firefox"
    );
    assert_eq!(
        program("avahi-daemon", "avahi-daemon: running\0"),
        "avahi-daemon"
    );
    assert_eq!(program("kthreadd", ""), "kthreadd");
}
