mod history;
mod presence;

use protocol::chat_presence::PresenceList;
use protocol::global_matchmaker::GlobalChatMessageType;
use protocol::user_identity::NodeIdentity;
use protocol::ReceivedMessage;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use tui_big_text::{BigText, PixelSize};

use super::ChatPageState;

fn draw_loading(frame: &mut Frame, data: &str) {
    let widget = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(Style::new().blue())
        .lines(vec![data.to_string().blue().into()])
        .centered()
        .build();
    frame.render_widget(widget, frame.area());
}

pub fn draw_chat(frame: &mut Frame, data: &ChatPageState) {
    match data {
        ChatPageState::ChatLoading { message } => draw_loading(frame, message),
        ChatPageState::ChatLoaded {
            own_identity,
            presence,
            msg_history,
        } => draw_chat_ui(frame, own_identity, presence, msg_history),
    }
}

fn draw_chat_ui(
    frame: &mut Frame,
    own_identity: &NodeIdentity,
    presence: &PresenceList<GlobalChatMessageType>,
    msg_history: &Vec<ReceivedMessage<GlobalChatMessageType>>,
) {
    use Constraint::{Fill, Length, Min};
    let vertical = Layout::vertical([Length(3), Min(0)]);
    let [title_area, main_area] = vertical.areas(frame.area());
    let horizontal = Layout::horizontal([Fill(1), Fill(2)]);
    let [left_area, right_area] = horizontal.areas(main_area);

    let title_bar = Block::bordered().title("Global Chat");
    let title_bar = Paragraph::new(own_identity.nickname())
        .block(title_bar)
        .centered();

    frame.render_widget(title_bar, title_area);
    presence::draw_presence_list(frame, left_area, presence);
    history::draw_chat_messages(frame, right_area, msg_history, own_identity);
}
