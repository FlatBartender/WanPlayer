#![windows_subsystem = "windows"]

use std::sync::Arc;

mod gensokyo_radio;
mod pipeline;
mod executor;

#[derive(Debug, Clone)]
enum PlayerMessage {
    Play,
    Pause,
    VolumeChanged(u8),
    AlbumArt(Option<Vec<u8>>),
    SongInfo(gensokyo_radio::GRApiAnswer),
    IncrementElapsed,
}

#[derive(PartialEq, Eq)]
enum PlayerStatus {
    Playing,
    Paused,
}

const PLAY_SVG: &str = include_str!("resources/play.svg");
const PAUSE_SVG: &str = include_str!("resources/pause.svg");
const FONT: &[u8] = include_bytes!("resources/NotoSansSC-Regular.otf");
const NO_IMAGE: &[u8] = include_bytes!("resources/gr-logo-placeholder.png");

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
    album_image: Option<Vec<u8>>,
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

        let commands = vec![
            Command::perform(async move { fut_api_client.get_song_info().await }, PlayerMessage::SongInfo),
            Command::perform(async move { tokio::time::sleep(std::time::Duration::from_secs(1)).await }, |_| {PlayerMessage::IncrementElapsed})
        ];

        (
            Player {
                player_status,
                player_tx,
                api_client,
                album_image: None,
                volume: DEFAULT_VOLUME,
                current_song_info: None,

                play_pause_state: widget::button::State::new(),
                volume_slider_state: widget::slider::State::new(),
            },
            Command::batch(commands)
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
            PlayerMessage::AlbumArt(opt_art) => {
                self.album_image = opt_art;
                Command::none()
            },
            PlayerMessage::SongInfo(song_info) => {
                self.current_song_info = Some(song_info.clone());
                let fut_api_client = self.api_client.clone();
                Command::perform(async move {
                    fut_api_client.get_album_image(&song_info).await
                }, PlayerMessage::AlbumArt)
            },
            PlayerMessage::IncrementElapsed => {
                let mut commands = Vec::with_capacity(2);
                if let Some(ref mut info) = self.current_song_info {
                    info.songtimes.played += 1;
                    if info.songtimes.played == info.songtimes.duration {
                        let fut_api_client = self.api_client.clone();
                        commands.push(Command::perform(async move { fut_api_client.get_song_info().await }, PlayerMessage::SongInfo))
                    }
                }
                commands.push(Command::perform(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await
                }, |_| {PlayerMessage::IncrementElapsed}));
                Command::batch(commands)
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let player = widget::Row::new();

        let type_column = widget::Column::new()
            .push(widget::Space::new(iced::Length::Shrink, iced::Length::Units(48)))
            .push(widget::Text::new("by").size(32).color([1.0, 1.0, 1.0, 0.5]))
            .push(widget::Text::new("album").size(32).color([1.0, 1.0, 1.0, 0.5]))
            .push(widget::Text::new("circle").size(32).color([1.0, 1.0, 1.0, 0.5]))
            .push(widget::Text::new("year").size(32).color([1.0, 1.0, 1.0, 0.5]))
            .align_items(iced::Align::End);

        let value_column = widget::Column::new();
        let value_column = if let Some(ref song_info) = self.current_song_info {
            value_column.push(widget::Text::new(&song_info.songinfo.title).size(48))
                .push(widget::Text::new(&song_info.songinfo.artist).size(32))
                .push(widget::Text::new(&song_info.songinfo.album).size(32))
                .push(widget::Text::new(&song_info.songinfo.circle).size(32))
                .push(widget::Text::new(&song_info.songinfo.year).size(32))
        } else {
            value_column.push(widget::Text::new("Fetching infos...").size(32))
        };

        let art_column = widget::Column::new();
        let art_column = match self.album_image {
            None => {
                let art = widget::Image::new(widget::image::Handle::from_memory(NO_IMAGE.to_vec()))
                    .height(iced::Length::Units(200))
                    .width(iced::Length::Units(200));
                art_column.push(art)
            },
            Some(ref art) => {
                let art = widget::Image::new(widget::image::Handle::from_memory(art.clone()))
                    .height(iced::Length::Units(200))
                    .width(iced::Length::Units(200));
                art_column.push(art)
            },
        };
        let elapsed_row = widget::Row::new();
        let elapsed_row = if let Some(ref song_info) = self.current_song_info {
             elapsed_row.push(widget::Text::new(format!("{}:{:02}", song_info.songtimes.played/60, song_info.songtimes.played%60)).width(iced::Length::Shrink))
                .push(widget::Text::new(format!("{}%", self.volume)).width(iced::Length::Fill).horizontal_alignment(iced::HorizontalAlignment::Center))
                .push(widget::Text::new(format!("{}:{:02}", song_info.songtimes.duration/60, song_info.songtimes.duration%60)).width(iced::Length::Shrink))
        } else {
            elapsed_row.push(widget::Text::new("0:00"))
                .push(widget::Text::new(format!("{}%", self.volume)).width(iced::Length::Fill).horizontal_alignment(iced::HorizontalAlignment::Center))
                .push(widget::Text::new("0:00"))
        };
        
        let (svg_source, button_message) = match self.player_status {
            PlayerStatus::Playing => (PAUSE_SVG, PlayerMessage::Pause),
            PlayerStatus::Paused => (PLAY_SVG, PlayerMessage::Play),
        };

        struct PlayPauseStyle;
        impl widget::button::StyleSheet for PlayPauseStyle {
            fn active(&self) -> widget::button::Style {
                widget::button::Style {
                    shadow_offset: iced::Vector::new(0.0, 0.0),
                    background: None,
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: iced::Color::new(0.0, 0.0, 0.0, 0.0),
                    text_color: iced::Color::new(0.0, 0.0, 0.0, 0.0),
                }
            }
        }

        let play_pause_svg = widget::Svg::new(widget::svg::Handle::from_memory(svg_source));
        let play_pause = widget::Button::new(&mut self.play_pause_state, play_pause_svg)
            .style(PlayPauseStyle)
            .on_press(button_message);

        struct SliderStyle;
        impl widget::slider::StyleSheet for SliderStyle {
            fn active(&self) -> widget::slider::Style {
                widget::slider::Style {
                    rail_colors: (iced::Color::new(1.0, 1.0, 1.0, 1.0), iced::Color::new(0.0, 0.0, 0.0, 0.0)),
                    handle: widget::slider::Handle {
                        shape: widget::slider::HandleShape::Circle {radius: 8.0},
                        color: iced::Color::new(1.0, 1.0, 1.0, 1.0),
                        border_width: 1.0,
                        border_color: iced::Color::new(0.0, 0.0, 0.0, 0.5),
                    },
                }
            }

            fn hovered(&self) -> widget::slider::Style {
                self.active()
            }

            fn dragging(&self) -> widget::slider::Style {
                self.active()
            }
        }

        let volume_slider = widget::Slider::new(&mut self.volume_slider_state, 0..=100, self.volume, PlayerMessage::VolumeChanged)
            .style(SliderStyle)
            .step(1);

        let controls = widget::Row::new()
            .push(play_pause)
            .push(volume_slider)
            .spacing(8)
            .align_items(iced::Align::Center);

        struct ProgressStyle;
        impl widget::progress_bar::StyleSheet for ProgressStyle {
            fn style(&self) -> widget::progress_bar::Style {
                widget::progress_bar::Style {
                    bar: iced::Background::Color(iced::Color::new(15.0 / 255.0, 135.0 / 255.0, 255.0 / 255.0, 1.0)),
                    background: iced::Background::Color(iced::Color::new(9.0 / 255.0, 81.0 / 255.0, 153.0 / 255.0, 1.0)),
                    border_radius: 0.0,
                }
            }
        }

        let art_column = art_column.push(if let Some(ref song_info) = self.current_song_info {
            widget::ProgressBar::new(0.0..=song_info.songtimes.duration as f32, song_info.songtimes.played as f32)
        } else {
            widget::ProgressBar::new(0.0..=100.0, 0.0)
        }.style(ProgressStyle).height(iced::Length::Units(8))).push(elapsed_row)
            .push(controls)
            .max_width(200);

        struct PlayerStyle;
        impl widget::container::StyleSheet for PlayerStyle {
            fn style(&self) -> widget::container::Style {
                widget::container::Style {
                    text_color: Some(iced::Color::new(1.0, 1.0, 1.0, 1.0)),
                    background: Some(iced::Background::Color(iced::Color::new(26.0 / 255.0, 21.0 / 255.0, 55.0 / 255.0, 1.0))),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: iced::Color::new(0.0, 0.0, 0.0, 0.0),
                }
            }
        }

        widget::Container::new(player
            .push(art_column)
            .push(type_column)
            .push(value_column)
            .spacing(8)
        ).style(PlayerStyle)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(8)
            .into()
    }
}


#[tokio::main]
async fn main() {
    let settings = Settings {
        default_font: Some(FONT),
        window: iced::window::Settings {
            size: (640, 294),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    };
    Player::run(settings).unwrap();
}
