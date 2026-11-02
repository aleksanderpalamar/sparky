use sparky_login_core::protocol::Power;

const SERVICE: &str = "org.freedesktop.login1";
const PATH: &str = "/org/freedesktop/login1";
const MANAGER: &str = "org.freedesktop.login1.Manager";

pub fn method(action: Power) -> &'static str {
    match action {
        Power::Shutdown => "PowerOff",
        Power::Reboot => "Reboot",
        Power::Suspend => "Suspend",
    }
}

pub async fn power(action: Power) -> Result<(), String> {
    let connection = zbus::Connection::system()
        .await
        .map_err(|error| format!("D-Bus do sistema: {error}"))?;
    let interactive = false;
    connection
        .call_method(
            Some(SERVICE),
            PATH,
            Some(MANAGER),
            method(action),
            &(interactive,),
        )
        .await
        .map(drop)
        .map_err(|error| format!("logind: {error}"))
}

#[cfg(test)]
mod tests {
    use sparky_login_core::protocol::Power;

    use super::method;

    #[test]
    fn cada_botao_chama_o_metodo_do_logind() {
        let metodos: Vec<&str> = Power::ALL.into_iter().map(method).collect();
        assert_eq!(metodos, vec!["PowerOff", "Reboot", "Suspend"]);
    }
}
