#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub full_name: String,
    pub home: String,
    pub shell: String,
}

pub fn humans(contents: &str) -> Vec<Account> {
    contents.lines().filter_map(account).filter(|account| (1000..65534).contains(&account.uid)).collect()
}

pub fn groups_of(contents: &str, account: &Account) -> Vec<String> {
    let mut found: Vec<String> = contents.lines().filter_map(|line| {
        let fields: Vec<_> = line.split(':').collect();
        let gid = fields.get(2)?.parse::<u32>().ok()?;
        let member = fields.get(3)?.split(',').any(|name| name.trim() == account.name);
        (gid == account.gid || member).then(|| fields[0].to_owned())
    }).collect();
    found.sort();
    found.dedup();
    found
}

fn account(line: &str) -> Option<Account> {
    let fields: Vec<_> = line.split(':').collect();
    Some(Account {
        name: fields.first()?.to_string(),
        uid: fields.get(2)?.parse().ok()?,
        gid: fields.get(3)?.parse().ok()?,
        full_name: fields.get(4)?.split(',').next().unwrap_or_default().to_owned(),
        home: fields.get(5)?.to_string(),
        shell: fields.get(6)?.to_string(),
    })
}
