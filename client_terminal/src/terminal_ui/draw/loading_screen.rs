use ratatui::style::{Style, Stylize};
use ratatui::Frame;
use tui_big_text::{BigText, PixelSize};

use super::super::app_state::LoadingWindowData;

pub fn draw_loading(frame: &mut Frame, data: &LoadingWindowData) {    
    let widget = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(Style::new().blue())
        .lines(vec![
            data.message.clone().blue().into(),
        ]
        )
        .centered()
        .build();
    frame.render_widget(widget, frame.area());


    // let widget = Block::bordered().title("Loading");
    // let widget = Paragraph::new(data.message.clone())
    //     .block(widget)
    //     .centered();
    // frame.render_widget(widget, frame.area());
}
