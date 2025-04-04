use protocol::chat_presence::PresenceList;
use protocol::global_matchmaker::GlobalChatMessageType;
use protocol::ReceivedMessage;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};

use super::app_state::{ChatWindowData, SpecificWindowData, WindowData};

pub fn draw_main(frame: &mut Frame, data: &WindowData) {
    match &data.data {
        SpecificWindowData::Loading(data) => {
            let widget = Block::bordered().title("Loading");
            let widget = Paragraph::new(data.message.clone())
                .block(widget)
                .centered();
            frame.render_widget(widget, frame.area());
        }
        SpecificWindowData::Chat(data) => {
            draw_chat(frame, data);
        }
    }
}

fn draw_chat(frame: &mut Frame, data: &ChatWindowData) {
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
    draw_presence_list(frame, left_area, &data.presence);
    draw_chat_messages(frame, right_area, &data.msg_history);
}

fn draw_presence_list(
    frame: &mut Frame,
    area: Rect,
    data: &PresenceList<GlobalChatMessageType>,
) {
    let block = Block::bordered().title("Presence List");
    let list_txt = format!("{:#?}", data);
    let presence_list = Paragraph::new(list_txt).block(block).centered();
    frame.render_widget(presence_list, area);
}

fn draw_chat_messages(
    frame: &mut Frame,
    area: Rect,
    data: &Vec<ReceivedMessage<GlobalChatMessageType>>,
) {
    let block = Block::bordered().title("Chat Messages");
    let messages_txt = format!("{:#?}", data);
    let messages = Paragraph::new(messages_txt).block(block).centered();
    frame.render_widget(messages, area);
}
