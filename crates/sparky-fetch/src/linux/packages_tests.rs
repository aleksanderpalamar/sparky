use crate::domain::status::{Reason, Status, Value};
use crate::linux::fake::Fake;
use crate::linux::host::Ran;
use crate::linux::packages;
use crate::linux::tally::{Tally, summary};

fn text(value: &str) -> Result<Value, Status> {
    Ok(Value::Text(value.to_owned()))
}

#[test]
fn pacman_conta_as_entradas_sem_o_arquivo_de_versao() {
    let host = Fake::default().dir(
        "/var/lib/pacman/local",
        &["ALPM_DB_VERSION", "zsh-5.9-5", "linux-6.17.1-1"],
    );
    assert_eq!(packages::collect(&host), text("2 (pacman)"));
}

#[test]
fn dpkg_conta_apenas_os_instalados() {
    let status = "Package: a\nStatus: install ok installed\n\nPackage: b\nStatus: deinstall ok config-files\n\nPackage: c\nStatus: install ok installed\n";
    let host = Fake::default().file("/var/lib/dpkg/status", status);
    assert_eq!(packages::collect(&host), text("2 (dpkg)"));
}

#[test]
fn rpm_conta_as_linhas_e_flatpak_soma_sistema_e_usuario() {
    let host = Fake::default()
        .command("rpm", Ran::Output("bash-5.2\nzsh-5.9\n\n".to_owned()))
        .var("HOME", "/home/u")
        .dir("/var/lib/flatpak/app", &["org.a"])
        .dir("/home/u/.local/share/flatpak/app", &["org.b", "org.c"]);
    assert_eq!(packages::collect(&host), text("2 (rpm), 3 (flatpak)"));
}

#[test]
fn gerenciadores_ausentes_ou_vazios_nao_aparecem() {
    let host = Fake::default()
        .dir("/var/lib/pacman/local", &["ALPM_DB_VERSION", "zsh-5.9-5"])
        .dir("/var/lib/flatpak/app", &[])
        .command("rpm", Ran::Output(String::new()));
    assert_eq!(packages::collect(&host), text("1 (pacman)"));
    assert_eq!(
        packages::collect(&Fake::default()),
        Err(Status::unavailable(Reason::Missing))
    );
}

#[test]
fn falhas_so_aparecem_quando_nada_foi_contado() {
    let lento = Fake::default().command("rpm", Ran::TimedOut);
    assert!(matches!(
        packages::collect(&lento),
        Err(Status::Failed { .. })
    ));
    let parcial = Fake::default()
        .command("rpm", Ran::Failed("erro".to_owned()))
        .dir("/var/lib/pacman/local", &["zsh-5.9-5"]);
    assert_eq!(packages::collect(&parcial), text("1 (pacman)"));
}

#[test]
fn contagens_parciais_sao_combinadas_com_prioridade_para_presentes() {
    let falha = Tally::Broken(Status::failed("x"));
    assert_eq!(
        Tally::Present(2).merge(Tally::Present(3)),
        Tally::Present(5)
    );
    assert_eq!(falha.clone().merge(Tally::Present(1)), Tally::Present(1));
    assert_eq!(Tally::Absent.merge(falha.clone()), falha);
    assert_eq!(Tally::Absent.merge(Tally::Absent), Tally::Absent);
    assert_eq!(
        summary(&[("a", Tally::Absent), ("b", falha.clone())]),
        Err(Status::failed("x"))
    );
}
