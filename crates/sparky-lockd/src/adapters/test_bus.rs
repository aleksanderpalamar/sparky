use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

use zbus::Connection;

pub struct Barramento {
    processo: Child,
    pub endereco: String,
}

impl Barramento {
    pub fn iniciar() -> Self {
        let mut processo = Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--nopidfile", "--print-address=1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|error| panic!("{error}"));
        let saida = processo
            .stdout
            .take()
            .unwrap_or_else(|| panic!("sem saída"));
        let mut endereco = String::new();
        let _ = BufReader::new(saida).read_line(&mut endereco);
        Self {
            processo,
            endereco: endereco.trim().to_owned(),
        }
    }

    pub async fn conectar(&self) -> Connection {
        zbus::connection::Builder::address(self.endereco.as_str())
            .map(|builder| builder.build())
            .unwrap_or_else(|error| panic!("{error}"))
            .await
            .unwrap_or_else(|error| panic!("{error}"))
    }
}

impl Drop for Barramento {
    fn drop(&mut self) {
        let _ = self.processo.kill();
        let _ = self.processo.wait();
    }
}
