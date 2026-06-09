use ratatui::{
    Frame,
    text::{Line, Text},
};

use crate::model::Model;

pub fn view(model: &Model, frame: &mut Frame) {
    let mut lines = vec![
        Line::from("Press q to quit"),
        Line::from("Press Space to toggle note"),
    ];
    if model.is_on {
        lines.push(Line::from("Note playing..."));
    }
    let text = Text::from(lines);
    frame.render_widget(text, frame.area());
}
