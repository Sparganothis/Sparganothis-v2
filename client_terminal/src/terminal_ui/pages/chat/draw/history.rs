use protocol::global_matchmaker::GlobalChatMessageType;
use protocol::user_identity::NodeIdentity;
use protocol::{datetime_now, ReceivedMessage};
use ratatui::layout::{Rect, Layout, Direction, Constraint};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use ratatui::widgets::Block;

pub fn draw_chat_messages(
    frame: &mut Frame,
    area: Rect,
    data: &Vec<ReceivedMessage<GlobalChatMessageType>>,
    own_identity: &NodeIdentity,
) {
    let mut current_y = area.y;
    let mut current_height = 0;

    for msg in data {
        let now = datetime_now();
        let elapsed = now
            .signed_duration_since(msg._received_timestamp)
            .abs()
            .num_seconds();
        let elapsed_txt = if elapsed < 60 {
            format!("{}s ago", elapsed)
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else {
            format!("{}h ago", elapsed / 3600)
        };

        // Determine alignment and color based on sender
        let alignment = if msg.from.user_id() == own_identity.user_id() {
            ratatui::layout::Alignment::Right
        } else {
            ratatui::layout::Alignment::Left
        };
        let color = msg.from.rgb_color();
        let color = Color::Rgb(color.0, color.1, color.2);

        // Create message box
        let box_style = Style::default().fg(color);
        let box_border = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .style(box_style)
            .title_alignment(alignment);

        // Create header with nickname and timestamp
        let header = Line::from(vec![
            Span::styled(msg.from.nickname(), Style::default().fg(color)),
            Span::raw(" "),
            Span::raw(elapsed_txt),
        ]);

        // Split message into lines if it's too long
        let message_lines: Vec<Line> = msg
            .message
            .split('\n')
            .map(|line| Line::from(vec![Span::raw(line)]))
            .collect();

        // Calculate message height (header + message lines + 2 for borders)
        let message_height = 2 + message_lines.len() + 1; // +2 for borders, +1 for header

        // Create message area
        let message_area = Rect {
            x: area.x,
            y: current_y + current_height,
            width: area.width,
            height: message_height as u16,
        };

        // Create message text
        let mut lines = Vec::new();
        lines.push(header);
        lines.extend(message_lines);

        let text = Text::from(lines);
        let message = Paragraph::new(text)
            .block(box_border)
            .alignment(alignment)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(message, message_area);

        // Update position for next message
        current_height += message_height as u16;
    }
}
