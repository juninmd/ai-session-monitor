use super::*;

#[test]
fn only_the_left_button_drives_the_flyout() {
    assert_eq!(gesture(MouseButton::Left, MouseButtonState::Down), Some(Gesture::Press));
    assert_eq!(gesture(MouseButton::Left, MouseButtonState::Up), Some(Gesture::Release));
    for button in [MouseButton::Right, MouseButton::Middle] {
        for state in [MouseButtonState::Down, MouseButtonState::Up] {
            assert_eq!(gesture(button, state), None, "{button:?} {state:?}");
        }
    }
}

#[test]
fn every_menu_entry_has_an_action_and_a_portuguese_label() {
    let menu = menu_entries(Language::PtBr);
    let actions: Vec<Option<Action>> = menu.iter().map(|(id, _)| action(id)).collect();
    assert_eq!(actions, [Some(Action::Open), Some(Action::Refresh), Some(Action::Quit)]);
    let labels: Vec<&str> = menu.iter().map(|(_, label)| *label).collect();
    assert_eq!(labels, ["Abrir", "Atualizar agora", "Sair"]);
}

#[test]
fn english_is_the_default_menu_language() {
    let labels: Vec<&str> = menu_entries(Language::default()).iter().map(|(_, l)| *l).collect();
    assert_eq!(labels, ["Open", "Refresh now", "Quit"]);
}

#[test]
fn an_unknown_menu_id_does_nothing() {
    assert_eq!(action(""), None);
    assert_eq!(action("OPEN"), None);
    assert_eq!(action("quit "), None);
}
