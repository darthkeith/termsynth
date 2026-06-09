use ratatui::{text::Text, Frame};

pub fn view(frame: &mut Frame) {
    let text = Text::raw("Press q to quit");
    frame.render_widget(text, frame.area());
}
