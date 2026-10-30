use crate::app::state::Settings;
use crate::domain::network::Network;

impl Settings {
    pub fn save_network(&mut self) {
        let chosen = self.network.profiles.get(self.network.selected).cloned();
        if let Err(error) = self.network.save(self.services.commands.as_ref()) {
            self.fail(error);
            return;
        }
        let reapplied = chosen.as_ref().is_some_and(|profile| {
            self.services
                .commands
                .run("nmcli", &["device", "reapply", &profile.interface])
                .is_ok()
        });
        self.network = Network::load(self.services.commands.as_ref());
        if let Some(index) = chosen.and_then(|chosen| {
            self.network
                .profiles
                .iter()
                .position(|profile| profile.uuid == chosen.uuid)
        }) {
            self.network.select(index);
        }
        self.inform(if reapplied {
            "Configuração IPv4 aplicada".to_owned()
        } else {
            "Configuração IPv4 salva; reconecte a rede para aplicar".to_owned()
        });
    }
}
