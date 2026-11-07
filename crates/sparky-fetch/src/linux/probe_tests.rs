use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::field::Field;
use crate::domain::status::{Reason, Status, Value};
use crate::linux::fake::Fake;
use crate::linux::files;
use crate::linux::host::Host;
use crate::linux::probe::Probe;
use crate::sources::Source;

fn pronto(_: &dyn Host) -> Result<Value, Status> {
    Ok(Value::Text("ok".to_owned()))
}

fn ausente(_: &dyn Host) -> Result<Value, Status> {
    Err(Status::unavailable(Reason::Missing))
}

#[test]
fn sonda_responde_apenas_quando_o_campo_foi_pedido() {
    let probe = Probe::new(&Field::Os, Arc::new(Fake::default()), pronto);
    assert_eq!(probe.fields(), [Field::Os]);
    assert_eq!(probe.collect(&[Field::Cpu]), []);
    assert_eq!(
        probe.collect(&[Field::Cpu, Field::Os]),
        [(Field::Os, Status::text("ok"))]
    );
    let falta = Probe::new(&Field::Gpu, Arc::new(Fake::default()), ausente);
    assert_eq!(
        falta.collect(&[Field::Gpu]),
        [(Field::Gpu, Status::unavailable(Reason::Missing))]
    );
}

#[test]
fn cada_campo_linux_tem_uma_unica_fonte() {
    let sources = super::wired(Arc::new(Fake::default()));
    let fields: Vec<Field> = sources
        .iter()
        .flat_map(|source| source.fields().to_vec())
        .collect();
    let unique: HashSet<Field> = fields.iter().copied().collect();
    assert_eq!(fields.len(), unique.len());
    for field in [Field::Os, Field::Packages, Field::Icons, Field::Font] {
        assert!(fields.contains(&field), "{}", field.key());
    }
}

#[test]
fn primeiro_arquivo_existente_vence_e_erros_reais_interrompem() {
    let paths = [PathBuf::from("/a"), PathBuf::from("/b")];
    let host = Fake::default().file("/b", "b");
    assert_eq!(
        files::first(&host, &paths),
        Ok((PathBuf::from("/b"), "b".to_owned()))
    );
    let broken = Fake::default().broken("/a").file("/b", "b");
    assert!(matches!(
        files::first(&broken, &paths),
        Err(Status::Failed { .. })
    ));
    assert_eq!(
        files::first(&Fake::default(), &paths),
        Err(Status::unavailable(Reason::Missing))
    );
}

#[test]
fn diretorio_do_usuario_ignora_xdg_relativo() {
    let host = Fake::default()
        .var("HOME", "/h")
        .var("XDG_CONFIG_HOME", "rel");
    assert_eq!(
        files::home(&host, "XDG_CONFIG_HOME", ".config"),
        Some(PathBuf::from("/h/.config"))
    );
    assert_eq!(
        files::home(&Fake::default(), "XDG_CONFIG_HOME", ".config"),
        None
    );
}
