use rodio::{
    DeviceSinkBuilder, Player, source::SineWave, stream::MixerDeviceSink,
};

const FREQ: f32 = 440.0;

pub struct Audio {
    _handle: MixerDeviceSink,
    player: Player,
}

pub enum Command {
    PlayNote,
    StopNote,
    None,
}

impl Audio {
    pub fn new() -> Self {
        let _handle = DeviceSinkBuilder::open_default_sink()
            .expect("failed to open default audio stream");
        let player = Player::connect_new(&_handle.mixer());
        let source = SineWave::new(FREQ);
        player.pause();
        player.append(source);
        Self { _handle, player }
    }
}

pub fn execute_command(command: Command, audio: &Audio) {
    match command {
        Command::PlayNote => audio.player.play(),
        Command::StopNote => audio.player.pause(),
        Command::None => (),
    }
}
