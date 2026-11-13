use std::time::{Duration, SystemTime};

use super::entry::{Entry, Kind, Meta};
use super::order::{Order, SortDirection, SortKey, sort};

fn item(name: &str, kind: Kind, size: u64, seconds: u64) -> Entry {
    Entry {
        name: name.to_owned(),
        kind,
        meta: Meta::Known {
            size,
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)),
        },
    }
}

fn nomes(entries: &[Entry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.name.as_str()).collect()
}

fn amostra() -> Vec<Entry> {
    vec![
        item("b.txt", Kind::File, 300, 10),
        item("Zeta", Kind::Directory, 4096, 5),
        item("a 10.png", Kind::File, 100, 30),
        item("a 2.jpg", Kind::File, 200, 20),
        item("alfa", Kind::Directory, 4096, 50),
        item("leia-me", Kind::File, 100, 40),
    ]
}

fn ordenado(order: Order) -> Vec<String> {
    let mut entries = amostra();
    sort(&mut entries, order);
    nomes(&entries).into_iter().map(str::to_owned).collect()
}

#[test]
fn padrao_e_nome_natural_com_pastas_antes() {
    assert_eq!(
        ordenado(Order::default()),
        ["alfa", "Zeta", "a 2.jpg", "a 10.png", "b.txt", "leia-me"]
    );
}

#[test]
fn tamanho_ordena_arquivos_e_empata_pelo_nome() {
    let order = Order::new(SortKey::Size, SortDirection::Ascending);
    assert_eq!(
        ordenado(order),
        ["alfa", "Zeta", "a 10.png", "leia-me", "a 2.jpg", "b.txt"]
    );
}

#[test]
fn decrescente_inverte_so_os_itens_e_mantem_pastas_no_topo() {
    let order = Order::new(SortKey::Modified, SortDirection::Descending);
    assert_eq!(
        ordenado(order),
        ["alfa", "Zeta", "leia-me", "a 10.png", "a 2.jpg", "b.txt"]
    );
}

#[test]
fn tipo_agrupa_pela_extensao_sem_extensao_primeiro() {
    let order = Order::new(SortKey::Type, SortDirection::Ascending);
    assert_eq!(
        ordenado(order),
        ["alfa", "Zeta", "leia-me", "a 2.jpg", "a 10.png", "b.txt"]
    );
}

#[test]
fn nome_decrescente_inverte_a_ordem_natural() {
    let order = Order::new(SortKey::Name, SortDirection::Descending);
    assert_eq!(
        ordenado(order),
        ["Zeta", "alfa", "leia-me", "b.txt", "a 10.png", "a 2.jpg"]
    );
}

#[test]
fn tamanho_desconhecido_vai_para_o_fim() {
    let mut entries = vec![
        Entry {
            name: "pendente".to_owned(),
            kind: Kind::File,
            meta: Meta::Pending,
        },
        item("grande", Kind::File, 900, 1),
    ];
    sort(
        &mut entries,
        Order::new(SortKey::Size, SortDirection::Descending),
    );
    assert_eq!(nomes(&entries), ["grande", "pendente"]);
}

#[test]
fn trocar_de_criterio_percorre_todos_e_inverter_alterna() {
    let mut order = Order::default();
    let mut vistos = Vec::new();
    for _ in 0..4 {
        order = order.next_key();
        vistos.push(order.key);
    }
    assert_eq!(
        vistos,
        [
            SortKey::Size,
            SortKey::Modified,
            SortKey::Type,
            SortKey::Name
        ]
    );
    assert_eq!(order.reversed().direction, SortDirection::Descending);
    assert_eq!(order.reversed().reversed(), order);
    assert!(!Order::default().needs_details());
    assert!(Order::new(SortKey::Size, SortDirection::Ascending).needs_details());
    assert!(!Order::new(SortKey::Type, SortDirection::Ascending).needs_details());
}

#[test]
fn escolher_o_mesmo_criterio_inverte_e_outro_comeca_crescente() {
    let tamanho = Order::new(SortKey::Size, SortDirection::Ascending);
    assert_eq!(
        tamanho.choose(SortKey::Size).direction,
        SortDirection::Descending
    );
    assert_eq!(
        tamanho.reversed().choose(SortKey::Name),
        Order::new(SortKey::Name, SortDirection::Ascending)
    );
}
