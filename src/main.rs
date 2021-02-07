mod gensokyo_radio;
mod pipeline;
mod executor;
mod widget;

#[derive(Debug, Clone)]
enum PlayerMessage {
    Play,
    Pause,
    VolumeChanged,
}

#[derive(PartialEq, Eq)]
enum PlayerStatus {
    Playing,
    Paused,
}

const PLAY_SVG: &str = include_str!("resources/play.svg");
const PAUSE_SVG: &str = include_str!("resources/pause.svg");

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
}

use iced_native::{
    event,
    mouse,
};

impl Application for Player {

    type Executor = executor::TokioExecutor;
    type Message = PlayerMessage;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let player_tx = pipeline::setup_pipeline();
        
        let player_status = PlayerStatus::Paused;
        let api_client = gensokyo_radio::ApiClient::new();

        (
            Player {
                player_status,
                api_client,
                player_tx,
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
            _ => unimplemented!()
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let play_pause = {
            let (event, message, svg_source) = match self.player_status {
                PlayerStatus::Playing => {
                    (
                        widget::Event::Mouse(widget::mouse::Event::ButtonPressed(widget::mouse::Button::Left)),
                        PlayerMessage::Pause,
                        PAUSE_SVG,
                    )
                },
                PlayerStatus::Paused => {
                    (
                        widget::Event::Mouse(widget::mouse::Event::ButtonPressed(widget::mouse::Button::Left)),
                        PlayerMessage::Play,
                        PLAY_SVG,
                    )
                }
            };

            let svg = iced_widget::Svg::new(iced_widget::svg::Handle::from_memory(svg_source));
            widget::AddEventListener::new(svg)
                .add_event_listener(event, message)
        };

        play_pause.into()
    }
}


#[tokio::main]
async fn main() {
    Player::run(Settings::default()).unwrap();
}
