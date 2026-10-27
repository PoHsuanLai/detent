use super::*;

fn form(name: &str, security: HiddenSecurity, username: &str, password: &str) -> HiddenForm {
    HiddenForm {
        name: name.to_owned(),
        security,
        username: username.to_owned(),
        password: password.to_owned(),
    }
}

#[test]
fn a_hidden_network_form_is_complete_when_its_security_has_what_it_needs() {
    let long_name = "n".repeat(33);
    let cases = [
        ("no name", form("", HiddenSecurity::Open, "", ""), None),
        (
            "a name too long for the air",
            form(&long_name, HiddenSecurity::Open, "", ""),
            None,
        ),
        (
            "an open one",
            form("Back Room", HiddenSecurity::Open, "", ""),
            Some(HiddenLogin::Open),
        ),
        (
            "a short password",
            form("Back Room", HiddenSecurity::Personal, "", "short"),
            None,
        ),
        (
            "wpa2 or wpa3",
            form("Back Room", HiddenSecurity::Personal, "", "correct horse"),
            Some(HiddenLogin::Personal(Passphrase::new("correct horse"))),
        ),
        (
            "wpa3 only",
            form(
                "Back Room",
                HiddenSecurity::PersonalWpa3,
                "",
                "correct horse",
            ),
            Some(HiddenLogin::PersonalWpa3(Passphrase::new("correct horse"))),
        ),
        (
            "an enterprise one without a username",
            form("Back Room", HiddenSecurity::Enterprise, "", "secret"),
            None,
        ),
        (
            "an enterprise one",
            form("Back Room", HiddenSecurity::Enterprise, "ada", "secret"),
            WifiLogin::new("ada", "secret").map(HiddenLogin::Enterprise),
        ),
    ];
    for (case, filled, login) in cases {
        assert_eq!(filled.network().map(|n| n.login), login, "{case}");
    }
}

#[test]
fn a_hidden_network_is_joined_from_its_sheet_and_a_refusal_is_said_not_asked_again() {
    let (open, _) = wifi_step(ready_with(world()), WifiInput::OpenOther);
    assert_eq!(page_of(&open).asking, WifiAsking::Other);
    let network = form("Back Room", HiddenSecurity::Personal, "", "correct horse")
        .network()
        .expect("complete");
    let (next, effects) = wifi_step(open, WifiInput::JoinOther(network.clone()));
    assert_eq!(effects, [WifiEffect::JoinHidden(network)]);
    assert_eq!(page_of(&next).asking, WifiAsking::Nothing);
    assert_eq!(page_of(&next).joining, Some(ssid("Back Room")));
    let (after, effects) = wifi_step(
        next,
        WifiInput::Joined(ssid("Back Room"), Err(JoinFailure::WrongPassword)),
    );
    assert_eq!(effects, [WifiEffect::CouldNotJoin]);
    assert_eq!(page_of(&after).asking, WifiAsking::Nothing);
    assert_eq!(page_of(&after).joining, None);
}

#[test]
fn a_form_never_prints_its_password() {
    let shown = format!(
        "{:?}",
        form("Back Room", HiddenSecurity::Personal, "", "correct horse")
    );
    assert!(!shown.contains("correct horse"), "{shown}");
}
