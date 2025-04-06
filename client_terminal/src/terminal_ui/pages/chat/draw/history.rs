use protocol::global_matchmaker::GlobalChatMessageType;
use protocol::user_identity::NodeIdentity;
use protocol::{datetime_now, ReceivedMessage};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tui_widget_list::{ListBuilder, ListState, ListView};

fn word_wrap(text: &str, width: u16) -> String {
    let width = width as usize;
    let mut result = String::new();
    let mut current_line = String::new();

    // First collapse all newlines into spaces
    let text = text.replace('\n', " ");

    for word in text.split_whitespace() {
        // If adding this word would exceed the width
        if current_line.len() + word.len() + 1 > width {
            if !current_line.is_empty() {
                // Add the current line to result
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
            }

            // If the word itself is longer than width, break it
            if word.len() > width {
                let mut remaining = word;
                while !remaining.is_empty() {
                    let chunk = if remaining.len() > width {
                        &remaining[..width]
                    } else {
                        remaining
                    };
                    result.push_str(chunk);
                    result.push('\n');
                    remaining = &remaining[chunk.len()..];
                }
                continue;
            }
        }

        // Add word to current line
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    // Add any remaining text
    if !current_line.is_empty() {
        result.push_str(&current_line);
    }
    result
}

struct ChatMessage {
    message: Paragraph<'static>,
    alignment: Alignment,
    height: u16,
}

impl ChatMessage {
    fn new(
        msg: &ReceivedMessage<GlobalChatMessageType>,
        own_identity: &NodeIdentity,
        width: u16,
    ) -> Self {
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
            Alignment::Right
        } else {
            Alignment::Left
        };
        let color = msg.from.rgb_color();
        let color = Color::Rgb(color.0, color.1, color.2);

        // Create message box
        let box_style = Style::default().fg(color);
        let box_border = Block::default()
            .borders(Borders::ALL)
            .style(box_style)
            .title_alignment(alignment);

        // Create header with nickname and timestamp
        let header_left =
            Span::styled(msg.from.nickname(), Style::default().fg(color))
                .into_left_aligned_line();
        let header_right = Span::raw(elapsed_txt).into_right_aligned_line();

        let msg_reflow = word_wrap(&msg.message, width);
        // Split message into lines if it's too long
        let message_lines: Vec<Line> = msg_reflow
            .split('\n')
            .map(|line| Line::from(vec![Span::raw(line.to_string())]))
            .collect();

        // Create message text
        let mut lines = Vec::new();
        // lines.push(header);
        lines.extend(message_lines);

        let height = lines.len() as u16 + 2;
        let text = Text::from(lines);
        let message = Paragraph::new(text)
            .block(box_border.title_top(header_left).title_top(header_right))
            .alignment(alignment)
            .wrap(Wrap { trim: true });

        Self {
            message,
            alignment,
            height,
        }
    }
}

impl Widget for ChatMessage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate 75% width
        let width = (area.width as f32 * 0.75) as u16;
        let x_offset = if self.alignment == Alignment::Right {
            area.right() - width
        } else {
            area.left()
        };

        // Create new area with 75% width
        let new_area = Rect {
            x: x_offset,
            y: area.y,
            width,
            height: area.height,
        };

        self.message.render(new_area, buf);
    }
}

pub fn draw_chat_messages(
    frame: &mut Frame,
    area: Rect,
    data: &Vec<ReceivedMessage<GlobalChatMessageType>>,
    own_identity: &NodeIdentity,
    scroll_position: usize,
) {
    let builder = ListBuilder::new(|context| {
        let msg = &data[context.index];
        let width = (area.width as f32 * 0.75) as u16;
        let chat_message = ChatMessage::new(msg, own_identity, width - 2);

        // Calculate the height needed for this message
        let message_height = chat_message.height;

        (chat_message, message_height as u16)
    });

    let list = ListView::new(builder, data.len());
    let mut state = ListState::default();
    let scroll_position = if data.is_empty() {
        None
    } else {
        Some(if scroll_position < data.len() {
            scroll_position
        } else {
            data.len() - 1
        })
    };
    state.select(scroll_position);

    list.render(area, frame.buffer_mut(), &mut state);
}
