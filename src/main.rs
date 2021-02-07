mod gensokyo_radio;
mod pipeline;
mod executor;

#[derive(Debug, Clone)]
enum PlayerMessage {
    Play,
    Pause,
    VolumeChanged(u8),
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
    widget as iced_widget,
};

struct Player {
    player_status: PlayerStatus,
    player_tx: std::sync::mpsc::Sender<pipeline::PlayerControl>,
    api_client: gensokyo_radio::ApiClient,
    volume: u8,
    
    play_pause_state: iced_widget::button::State,
    volume_slider_state: iced_widget::slider::State,
}

impl Application for Player {

    type Executor = executor::TokioExecutor;
    type Message = PlayerMessage;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let player_tx = pipeline::setup_pipeline();
        
        let player_status = PlayerStatus::Paused;
        let api_client = gensokyo_radio::ApiClient::new();

        player_tx.send(pipeline::PlayerControl::Volume(DEFAULT_VOLUME)).expect("Failed to set initial volume");

        (
            Player {
                player_status,
                api_client,
                player_tx,
                volume: DEFAULT_VOLUME,

                play_pause_state: iced_widget::button::State::new(),
                volume_slider_state: iced_widget::slider::State::new(),
            },
            iced::Command::none()
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
            },
            PlayerMessage::Pause => {
                self.player_tx.send(PlayerControl::Pause).expect("Failed to send pause command to Player");
                self.player_status = PlayerStatus::Paused;
            },
            PlayerMessage::VolumeChanged(volume) => {
                self.player_tx.send(PlayerControl::Volume(volume)).expect("Failed to send volume command to Player");
                self.volume = volume;
            },
            _ => unimplemented!()
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let (svg_source, button_message) = match self.player_status {
                PlayerStatus::Playing => (PAUSE_SVG, PlayerMessage::Pause),
                PlayerStatus::Paused => (PLAY_SVG, PlayerMessage::Play),
            };

        let play_pause_svg = iced_widget::Svg::new(iced_widget::svg::Handle::from_memory(svg_source));
        let play_pause = iced_widget::Button::new(&mut self.play_pause_state, play_pause_svg)
            .on_press(button_message);

        let volume_slider = iced_widget::Slider::new(&mut self.volume_slider_state, 0..=100, self.volume, PlayerMessage::VolumeChanged)
            .step(1);

        let controls = iced_widget::Row::new()
            .push(play_pause)
            .push(volume_slider);

        controls.into()
    }
}


#[tokio::main]
async fn main() {
    Player::run(Settings::default()).unwrap();
}
