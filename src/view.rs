use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Text},
};

use crate::model::{Model, Param};

const NONE: &str = "None";

pub fn view(model: &Model, frame: &mut Frame) {
    let style = |param| {
        if model.selected == param {
            Style::new().add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        }
    };
    let midi_port_name = match &model.port_name {
        Some(name) => &name,
        None => NONE,
    };
    let midi_text = match &model.last_midi {
        Some((timestamp, bytes)) => {
            format!("Timestamp: {timestamp}, Bytes: {bytes:?}")
        }
        None => NONE.to_string(),
    };
    let note = match &model.note_state {
        Some(n) => n.to_string(),
        None => NONE.to_string(),
    };
    let lines = vec![
        Line::from("q to quit / Space to toggle test note"),
        Line::from(format!("Waveform: {}", model.waveform.name())),
        Line::from(format!(" Cutoff: {:.0} Hz", model.cutoff))
            .style(style(Param::Cutoff)),
        Line::from(format!(" Attack: {:.3} s", model.adsr.attack))
            .style(style(Param::Attack)),
        Line::from(format!("  Decay: {:.3} s", model.adsr.decay))
            .style(style(Param::Decay)),
        Line::from(format!("Sustain: {:.2}", model.adsr.sustain))
            .style(style(Param::Sustain)),
        Line::from(format!("Release: {:.3} s", model.adsr.release))
            .style(style(Param::Release)),
        Line::from(format!("Midi Input: {midi_port_name}")),
        Line::from(format!("Last MIDI: {midi_text}")),
        Line::from(format!("Note Playing: {note}")),
    ];
    let text = Text::from(lines);
    frame.render_widget(text, frame.area());
}
