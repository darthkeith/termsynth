use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Text},
};

use crate::model::{Model, Param};

pub fn view(model: &Model, frame: &mut Frame) {
    let style = |param| {
        if model.selected == param {
            Style::new().add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        }
    };
    let mut lines = vec![
        Line::from("Press q to quit"),
        Line::from("Press Space to toggle note"),
        Line::from(format!("Waveform: {}", model.waveform.name())),
        Line::from(format!(" Attack: {:.3} s", model.adsr.attack))
            .style(style(Param::Attack)),
        Line::from(format!("  Decay: {:.3} s", model.adsr.decay))
            .style(style(Param::Decay)),
        Line::from(format!("Sustain: {:.2}", model.adsr.sustain))
            .style(style(Param::Sustain)),
        Line::from(format!("Release: {:.3} s", model.adsr.release))
            .style(style(Param::Release)),
    ];
    if model.is_on {
        lines.push(Line::from("Note playing..."));
    }
    let text = Text::from(lines);
    frame.render_widget(text, frame.area());
}
