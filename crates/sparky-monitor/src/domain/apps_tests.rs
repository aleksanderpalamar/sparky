use super::apps::{AppEntry, group, program_of};
use super::usage::{ProcessRow, Usage};

fn entrada(id: &str, program: &str) -> AppEntry {
    AppEntry {
        id: id.to_owned(),
        name: format!("App {id}"),
        icon: Some(format!("{id}-icon")),
        program: program.to_owned(),
    }
}

fn processo(pid: u32, ppid: u32, program: &str, user: u32, cpu: f32) -> ProcessRow {
    ProcessRow {
        pid,
        ppid,
        name: program.to_owned(),
        program: program.to_owned(),
        user: Some(user),
        usage: Usage {
            cpu,
            memory: 100,
            ..Usage::default()
        },
    }
}

#[test]
fn programa_do_exec_ignora_variaveis_e_interpretadores() {
    assert_eq!(
        program_of("/usr/lib/firefox/firefox %u").as_deref(),
        Some("firefox")
    );
    assert_eq!(
        program_of("env GDK_BACKEND=x11 \"/opt/app/bin/app\" --x").as_deref(),
        Some("app")
    );
    assert_eq!(program_of("sh -c 'echo oi'"), None);
    assert_eq!(program_of("flatpak run org.gimp.GIMP"), None);
    assert_eq!(program_of(""), None);
}

#[test]
fn processos_filhos_somam_no_aplicativo_mais_proximo() {
    let catalogo = [
        entrada("firefox", "firefox"),
        entrada("terminal", "sparky-terminal"),
    ];
    let processos = [
        processo(1, 0, "systemd", 0, 0.0),
        processo(10, 1, "sparky-terminal", 1000, 1.0),
        processo(11, 10, "zsh", 1000, 0.5),
        processo(12, 11, "firefox", 1000, 3.0),
        processo(13, 12, "Web Content", 1000, 2.0),
        processo(20, 1, "avahi-daemon", 0, 9.0),
    ];
    let apps = group(&processos, &catalogo, 1000);
    let resumo: Vec<_> = apps
        .iter()
        .map(|app| {
            (
                app.id.as_str(),
                app.pids.clone(),
                app.usage.cpu,
                app.usage.memory,
            )
        })
        .collect();
    assert_eq!(
        resumo,
        [
            ("firefox", vec![12, 13], 5.0, 200),
            ("terminal", vec![10, 11], 1.5, 200)
        ]
    );
    assert_eq!(apps[0].icon.as_deref(), Some("firefox-icon"));
}

#[test]
fn ciclo_de_pais_nao_trava_o_agrupamento() {
    let processos = [
        processo(5, 6, "a", 1000, 0.0),
        processo(6, 5, "b", 1000, 0.0),
    ];
    assert!(group(&processos, &[entrada("x", "x")], 1000).is_empty());
}
