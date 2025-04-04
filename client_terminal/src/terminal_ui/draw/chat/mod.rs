mod presence;
mod history;


use ratatui::widgets::Paragraph;
use ratatui::Frame;

use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};

use super::super::app_state::ChatWindowData;

pub fn draw_chat(frame: &mut Frame, data: &ChatWindowData) {
    use Constraint::{Fill, Length, Min};

    let vertical = Layout::vertical([Length(3), Min(0)]);
    let [title_area, main_area] = vertical.areas(frame.area());
    let horizontal = Layout::horizontal([Fill(1), Fill(2)]);
    let [left_area, right_area] = horizontal.areas(main_area);

    let title_bar = Block::bordered().title("Global Chat");
    let title_bar = Paragraph::new(data.own_identity.nickname())
        .block(title_bar)
        .centered();

    frame.render_widget(title_bar, title_area);
    presence::draw_presence_list(frame, left_area, &data.presence);
    history::draw_chat_messages(frame, right_area, &data.msg_history, &data.own_identity);
}
