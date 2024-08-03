use ratatui::widgets::ListState;

pub(super) fn default_list_state() -> ListState {
    let mut list_state = ListState::default();
    list_state.select_first();
    list_state
}
