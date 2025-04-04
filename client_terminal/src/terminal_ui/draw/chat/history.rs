use protocol::chat_presence::PresenceList;
use protocol::global_matchmaker::GlobalChatMessageType;
use protocol::{datetime_now, ReceivedMessage};
use protocol::user_identity::NodeIdentity;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Text, Line, Span};
use std::time::Duration;

use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
};

pub fn draw_chat_messages(
    frame: &mut Frame,
    area: Rect,
    data: &Vec<ReceivedMessage<GlobalChatMessageType>>,
    own_identity: &NodeIdentity,
) {
    let block = Block::bordered().title("Chat Messages");
    
    let mut lines = Vec::new();
    for msg in data {
        let now = datetime_now();
        let elapsed = now.signed_duration_since(msg._received_timestamp).abs().num_seconds();
        let elapsed_txt = if elapsed < 60 {
            format!("{}s ago", elapsed)
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else {
            format!("{}h ago", elapsed / 3600)
        };

        // Determine alignment and color based on sender
        let (alignment, color) = if msg.from.user_id() == own_identity.user_id() {
            (ratatui::layout::Alignment::Right, Color::Green)
        } else {
            (ratatui::layout::Alignment::Left, Color::Cyan)
        };

        // Create message box
        let box_style = Style::default().fg(color);
        let box_border = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .style(box_style);

        // Create header with nickname and timestamp
        let header = Line::from(vec![
            Span::styled(msg.from.nickname(), Style::default().fg(color)),
            Span::raw(" "),
            Span::raw(elapsed_txt),
        ]);

        // Split message into lines if it's too long
        let message_lines: Vec<Line> = msg.message
            .split('\n')
            .map(|line| Line::from(vec![Span::raw(line)]))
            .collect();

        // Add header and message lines
        lines.push(header);
        lines.extend(message_lines);
        lines.push(Line::from("")); // Add empty line between messages
    }

    let text = Text::from(lines);
    let messages = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(messages, area);
}
