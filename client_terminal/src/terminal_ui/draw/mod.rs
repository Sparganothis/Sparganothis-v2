mod loading_screen;
mod chat;

use ratatui::Frame;

use super::app_state::{SpecificWindowData, WindowData};

pub fn draw_main(frame: &mut Frame, data: &WindowData) {
    match &data.data {
        SpecificWindowData::Loading(data) => {
            loading_screen::draw_loading(frame, data);
        }
        
        SpecificWindowData::Chat(data) => {
            chat::draw_chat(frame, data);
        }
    }
}
