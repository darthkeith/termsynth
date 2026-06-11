use ratatui::{
    Frame,
    text::{Line, Text},
};

use crate::model::Model;

pub fn view(model: &Model, frame: &mut Frame) {
    let mut lines = vec![
        Line::from("Press q to quit"),
        Line::from("Press Space to toggle note"),
        Line::from(format!(" Attack: {:.3} s", model.adsr.attack)),
        Line::from(format!("  Decay: {:.3} s", model.adsr.decay)),
        Line::from(format!("Sustain: {:.2}", model.adsr.sustain)),
        Line::from(format!("Release: {:.3} s", model.adsr.release)),
    ];
    if model.is_on {
        lines.push(Line::from("Note playing..."));
    }
    let text = Text::from(lines);
    frame.render_widget(text, frame.area());
}
