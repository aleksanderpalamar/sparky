use crate::domain::system::power::Power;
use crate::domain::system::{Change, Control, Request, RowId, perform};
use crate::domain::value::Step;
use crate::ports::fake::FakeCommands;

const LISTING: &str =
    "* performance:\n    CpuDriver:\tamd_pstate\n\n  balanced:\n    CpuDriver:\tamd_pstate\n";
const BATTERY: &str =
    "  native-path: BAT0\n  state:               discharging\n    percentage:          87%\n";

fn desktop() -> FakeCommands {
    FakeCommands::new()
        .with("powerprofilesctl list", LISTING)
        .with(
            "upower -e",
            "/org/freedesktop/UPower/devices/DisplayDevice\n",
        )
}

fn laptop() -> FakeCommands {
    FakeCommands::new()
        .with("powerprofilesctl list", LISTING)
        .with(
            "upower -e",
            "/org/freedesktop/UPower/devices/battery_BAT0\n",
        )
        .with(
            "upower -i /org/freedesktop/UPower/devices/battery_BAT0",
            BATTERY,
        )
        .with(
            "brightnessctl -m -c backlight -l",
            "intel_backlight,backlight,600,45%,1200\n",
        )
}

#[test]
fn no_desktop_so_aparece_o_perfil() {
    let rows = Power::load(&desktop()).rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, RowId::PowerProfile);
    assert_eq!(rows[0].value, "Desempenho");
    assert_eq!(
        rows[0].control,
        Control::Options(vec!["Desempenho".to_owned(), "Equilibrado".to_owned()])
    );
}

#[test]
fn no_notebook_aparecem_brilho_e_bateria() {
    let rows = Power::load(&laptop()).rows();
    let ids: Vec<RowId> = rows.iter().map(|row| row.id).collect();
    assert_eq!(
        ids,
        [RowId::PowerProfile, RowId::Brightness, RowId::Battery]
    );
    assert_eq!(rows[1].value, "45%");
    assert_eq!(rows[2].value, "87% · descarregando");
}

#[test]
fn sem_o_daemon_de_perfis_a_linha_avisa() {
    let rows = Power::load(&FakeCommands::new()).rows();
    assert_eq!(rows[0].control, Control::Reading);
    assert_eq!(rows[0].value, "power-profiles-daemon indisponível");
}

#[test]
fn as_setas_trocam_o_perfil_dando_a_volta() {
    let power = Power::load(&desktop());
    assert_eq!(
        power.request(RowId::PowerProfile, Change::Step(Step::More)),
        Some(Request::SetProfile("balanced".to_owned()))
    );
    assert_eq!(
        power.request(RowId::PowerProfile, Change::Step(Step::Less)),
        Some(Request::SetProfile("balanced".to_owned()))
    );
}

#[test]
fn o_brilho_anda_de_cinco_em_cinco_sem_apagar_a_tela() {
    let power = Power::load(&laptop());
    let pedido = |step| power.request(RowId::Brightness, Change::Step(step));
    assert_eq!(
        pedido(Step::More),
        Some(Request::SetBrightness {
            device: "intel_backlight".to_owned(),
            percent: 50
        })
    );
    let escuro = Power {
        backlight: power.backlight.clone().map(|mut light| {
            light.percent = 3;
            light
        }),
        ..power.clone()
    };
    assert_eq!(
        escuro.request(RowId::Brightness, Change::Step(Step::Less)),
        Some(Request::SetBrightness {
            device: "intel_backlight".to_owned(),
            percent: 1
        })
    );
}

#[test]
fn a_bateria_so_se_le() {
    assert_eq!(
        Power::load(&laptop()).request(RowId::Battery, Change::Activate),
        None
    );
}

#[test]
fn os_pedidos_viram_os_comandos_do_sistema() {
    let commands = FakeCommands::new()
        .accepting("powerprofilesctl set balanced")
        .accepting("brightnessctl -d intel_backlight set 70%");
    assert_eq!(
        perform(&commands, &Request::SetProfile("balanced".to_owned())),
        Ok(())
    );
    assert_eq!(
        perform(
            &commands,
            &Request::SetBrightness {
                device: "intel_backlight".to_owned(),
                percent: 70
            }
        ),
        Ok(())
    );
    assert_eq!(
        commands.calls(),
        [
            "powerprofilesctl set balanced",
            "brightnessctl -d intel_backlight set 70%"
        ]
    );
}

#[test]
fn escolher_um_perfil_pelo_nome_mostrado_pede_a_troca() {
    let power = Power::load(&desktop());
    assert_eq!(
        power.request(RowId::PowerProfile, Change::Text("Equilibrado".to_owned())),
        Some(Request::SetProfile("balanced".to_owned()))
    );
    assert_eq!(
        power.request(RowId::PowerProfile, Change::Text("Desempenho".to_owned())),
        None
    );
    assert_eq!(
        power.request(RowId::PowerProfile, Change::Text("Turbo".to_owned())),
        None
    );
}

#[test]
fn brilho_aceita_o_valor_do_controle_deslizante_dentro_dos_limites() {
    let power = Power::load(&laptop());
    let brilho = |valor: &str| power.request(RowId::Brightness, Change::Text(valor.to_owned()));
    let pedido = |percent| {
        Some(Request::SetBrightness {
            device: "intel_backlight".to_owned(),
            percent,
        })
    };
    assert_eq!(brilho("70"), pedido(70));
    assert_eq!(brilho("0"), pedido(1));
    assert_eq!(brilho("45"), None);
    assert_eq!(brilho("muito"), None);
}
