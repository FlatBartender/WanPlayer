use std::sync::Arc;

mod gensokyo_radio;
mod pipeline;
mod executor;

#[derive(Debug, Clone)]
enum PlayerMessage {
    Play,
    Pause,
    VolumeChanged(u8),
    SongInfo(gensokyo_radio::GRApiAnswer),
}

#[derive(PartialEq, Eq)]
enum PlayerStatus {
    Playing,
    Paused,
}

const PLAY_SVG: &str = include_str!("resources/play.svg");
const PAUSE_SVG: &str = include_str!("resources/pause.svg");

const DEFAULT_VOLUME: u8 = 10;

use iced::{
    Application,
    Command,
    Element,
    Settings,
    widget,
};

struct Player {
    player_status: PlayerStatus,
    player_tx: std::sync::mpsc::Sender<pipeline::PlayerControl>,
    api_client: Arc<gensokyo_radio::ApiClient>,
    volume: u8,
    current_song_info: Option<gensokyo_radio::GRApiAnswer>,
    
    play_pause_state: widget::button::State,
    volume_slider_state: widget::slider::State,
}

impl Application for Player {

    type Executor = executor::TokioExecutor;
    type Message = PlayerMessage;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let player_tx = pipeline::setup_pipeline();
        
        let player_status = PlayerStatus::Paused;
        let api_client = Arc::new(gensokyo_radio::ApiClient::new());
        let fut_api_client = api_client.clone();

        player_tx.send(pipeline::PlayerControl::Volume(DEFAULT_VOLUME)).expect("Failed to set initial volume");

        (
            Player {
                player_status,
                player_tx,
                api_client,
                volume: DEFAULT_VOLUME,
                current_song_info: None,

                play_pause_state: widget::button::State::new(),
                volume_slider_state: widget::slider::State::new(),
            },
            Command::perform(async move { fut_api_client.get_song_info().await }, PlayerMessage::SongInfo)
        )
    }

    fn title(&self) -> String {
        String::from("Wan Player")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use pipeline::PlayerControl;

        match message {
            PlayerMessage::Play => {
                self.player_tx.send(PlayerControl::Play).expect("Failed to send play command to Player");
                self.player_status = PlayerStatus::Playing;
                Command::none()
            },
            PlayerMessage::Pause => {
                self.player_tx.send(PlayerControl::Pause).expect("Failed to send pause command to Player");
                self.player_status = PlayerStatus::Paused;
                Command::none()
            },
            PlayerMessage::VolumeChanged(volume) => {
                self.player_tx.send(PlayerControl::Volume(volume)).expect("Failed to send volume command to Player");
                self.volume = volume;
                Command::none()
            },
            PlayerMessage::SongInfo(song_info) => {
                let sleep_duration = song_info.songtimes.remaining + 1;
                self.current_song_info = Some(song_info);
                let fut_api_client = self.api_client.clone();
                Command::perform(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(sleep_duration as u64)).await;
                    fut_api_client.get_song_info().await
                }, PlayerMessage::SongInfo)
            }
            _ => unimplemented!()
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let infos = widget::Column::new();
        let infos = if let Some(ref song_info) = self.current_song_info {
            infos.push(widget::Text::new(&song_info.songinfo.title))
                .push(widget::Text::new(&song_info.songinfo.artist))
        } else {
            infos
        };

        let (svg_source, button_message) = match self.player_status {
                PlayerStatus::Playing => (PAUSE_SVG, PlayerMessage::Pause),
                PlayerStatus::Paused => (PLAY_SVG, PlayerMessage::Play),
            };

        let play_pause_svg = widget::Svg::new(widget::svg::Handle::from_memory(svg_source));
        let play_pause = widget::Button::new(&mut self.play_pause_state, play_pause_svg)
            .on_press(button_message);

        let volume_slider = widget::Slider::new(&mut self.volume_slider_state, 0..=100, self.volume, PlayerMessage::VolumeChanged)
            .step(1);

        let controls = widget::Row::new()
            .push(play_pause)
            .push(volume_slider);

        widget::Column::new()
            .push(infos)
            .push(controls)
            .into()
    }
}


#[tokio::main]
async fn main() {
    Player::run(Settings::default()).unwrap();
}
