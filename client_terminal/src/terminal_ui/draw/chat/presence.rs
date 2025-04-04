use protocol::chat_presence::{PresenceFlag, PresenceList};
use protocol::global_matchmaker::GlobalChatMessageType;
use protocol::ReceivedMessage;
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

fn truncate_string(s: &str, max_len: usize) -> String {
    s.split_whitespace()
        .map(|word| truncate_word(word, max_len))
        .collect::<Vec<String>>()
        .join(" ")
}

fn truncate_word(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

pub fn draw_presence_list(
    frame: &mut Frame,
    area: Rect,
    data: &PresenceList<GlobalChatMessageType>,
) {
    let block = Block::bordered().title("Presence List");
    
    let mut lines = Vec::new();
    for item in data {
        let status_color = match item.presence_flag {
            PresenceFlag::ACTIVE => Color::Green,
            PresenceFlag::IDLE => Color::Yellow,
            PresenceFlag::EXPIRED => Color::Red,
            PresenceFlag::UNCONFIRMED => Color::DarkGray,
        };

        let elapsed = item.last_seen.elapsed();
        let elapsed_txt = if elapsed < Duration::from_secs(60) {
            format!("{}s ago", elapsed.as_secs())
        } else if elapsed < Duration::from_secs(3600) {
            format!("{}m ago", elapsed.as_secs() / 60)
        } else {
            format!("{}h ago", elapsed.as_secs() / 3600)
        };

        let rtt_txt = item.rtt.map(|rtt| format!(" ({}ms)", rtt)).unwrap_or_default();
        
        let status = match item.presence_flag {
            PresenceFlag::ACTIVE => "Active",
            PresenceFlag::IDLE => "Idle",
            PresenceFlag::EXPIRED => "Expired",
            PresenceFlag::UNCONFIRMED => "Unconfirmed",
        };

        // First line: truncated name
        let truncated_name = truncate_string(&item.identity.nickname(), 15);
        let name_line = Line::from(vec![
            Span::styled(truncated_name, Style::default().fg(status_color)),
        ]);
        lines.push(name_line);

        // Second line: status info
        let info_line = Line::from(vec![
            Span::raw("  "), // Indent
            Span::raw(elapsed_txt),
            Span::raw(rtt_txt),
            Span::raw(" - "),
            Span::raw(status),
        ]);
        lines.push(info_line);
    }

    let text = Text::from(lines);
    let presence_list = Paragraph::new(text).block(block);
    frame.render_widget(presence_list, area);
}
