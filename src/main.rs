// Copyright 2021 Flat Bartender <flat.bartender@gmail.com>
// 
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
// 
//        http://www.apache.org/licenses/LICENSE-2.0
// 
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, windows_subsystem = "console")]

use std::sync::Arc;

mod gensokyo_radio;
mod pipeline;
mod executor;
mod ui;

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


enum DiscordControl {
    SongInfo(gensokyo_radio::GRApiAnswer),
}

use iced::{
    Application,
    Command,
    Element,
    Settings,
    widget,
};
use discord_game_sdk::*;

const DISCORD_CLIENT_ID: i64 = 808130280976023563;

struct Player {
    player_status: PlayerStatus,
    player_tx: tokio::sync::mpsc::UnboundedSender<pipeline::PlayerControl>,
    discord_tx: std::sync::mpsc::Sender<DiscordControl>,
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
        let (discord_tx, discord_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            struct DiscordEventHandler;
            impl EventHandler for DiscordEventHandler {}

            let mut discord = Discord::new(DISCORD_CLIENT_ID).expect("Failed to connect to Discord API");
            discord.register_launch_command("https://gensokyoradio.net/music/playing/").expect("Failed to register launch command");
            *discord.event_handler_mut() = Some(DiscordEventHandler);
            loop {
                let result = discord_rx.recv_timeout(std::time::Duration::from_secs(1));
                match result {
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => discord.run_callbacks().expect("Failed to run discord callbacks"),
                    Err(_) => panic!(),
                    Ok(DiscordControl::SongInfo(song_info)) => {
                        let mut activity = Activity::empty();
                        activity
                            .with_state("Listening to Gensokyo Radio")
                            .with_details(&format!("{} - {}", &song_info.songinfo.artist, &song_info.songinfo.title))
                            .with_start_time(song_info.songtimes.songstart as i64)
                            .with_end_time(song_info.songtimes.songend as i64)
                            .with_large_image_key("presence_image");
                        discord.update_activity(&activity, |_, result| {
                            if let Err(err) = result {
                                println!("Error: {}", err);
                            }
                        });
                    }
                }
            }
        });

        let commands = vec![
            Command::perform(async move { fut_api_client.get_song_info().await }, PlayerMessage::SongInfo),
            Command::perform(async move { tokio::time::sleep(std::time::Duration::from_secs(1)).await }, |_| {PlayerMessage::IncrementElapsed})
        ];

        (
            Player {
                player_status,
                player_tx,
                discord_tx,
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
                self.discord_tx.send(DiscordControl::SongInfo(song_info.clone())).expect("Failed to send song info to discord");
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

        let art_column =  {
            let album_image = widget::Image::new(widget::image::Handle::from_memory(
                    if let Some(ref art) =  self.album_image {
                        art.clone()
                    } else {
                        ui::NO_IMAGE.to_vec()
                    })).height(iced::Length::Units(200))
            .width(iced::Length::Units(200));

            let elapsed_row = widget::Row::new();
            let elapsed_row = if let Some(ref song_info) = self.current_song_info {
                elapsed_row.push(widget::Text::new(format!("{}:{:02}", song_info.songtimes.played/60, song_info.songtimes.played%60)).width(iced::Length::Shrink))
                    .push(widget::Text::new(format!("{}%", self.volume)).width(iced::Length::Fill).horizontal_alignment(iced::HorizontalAlignment::Center))
                    .push(widget::Text::new(format!("{}:{:02}", song_info.songtimes.duration/60, song_info.songtimes.duration%60)).width(iced::Length::Shrink))
            } else {
                elapsed_row.push(widget::Text::new("--:--"))
                    .push(widget::Text::new(format!("{}%", self.volume)).width(iced::Length::Fill).horizontal_alignment(iced::HorizontalAlignment::Center))
                    .push(widget::Text::new("--:--"))
            };

            let (svg_source, button_message) = match self.player_status {
                PlayerStatus::Playing => (ui::PAUSE_SVG, PlayerMessage::Pause),
                PlayerStatus::Paused => (ui::PLAY_SVG, PlayerMessage::Play),
            };

            let play_pause_svg = widget::Svg::new(widget::svg::Handle::from_memory(svg_source));
            let play_pause = widget::Button::new(&mut self.play_pause_state, play_pause_svg)
                .style(ui::PlayPauseStyle)
                .on_press(button_message);

            let volume_slider = widget::Slider::new(&mut self.volume_slider_state, 0..=100, self.volume, PlayerMessage::VolumeChanged)
                .style(ui::VolumeSliderStyle)
                .step(1);

            let controls = widget::Row::new()
                .push(play_pause)
                .push(volume_slider)
                .spacing(8)
                .align_items(iced::Align::Center);

            let progress_bar = if let Some(ref song_info) = self.current_song_info {
                widget::ProgressBar::new(0.0..=song_info.songtimes.duration as f32, song_info.songtimes.played as f32)
            } else {
                widget::ProgressBar::new(0.0..=100.0, 0.0)
            }.style(ui::SongProgressStyle).height(iced::Length::Units(8));

            widget::Column::new()
                .push(album_image)
                .push(progress_bar)
                .push(elapsed_row)
                .push(controls)
                .max_width(200)
        };
        let info_panel = {
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

            widget::Row::new()
                .push(type_column)
                .push(value_column)
                .spacing(8)
        };

        widget::Container::new(player
            .push(art_column)
            .push(info_panel)
            .spacing(8)
        ).style(ui::PlayerStyle)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(8)
            .into()
    }
}

const DEFAULT_VOLUME: u8 = 10;
const FONT: &[u8] = include_bytes!("resources/NotoSansSC-Regular.otf");

#[tokio::main]
async fn main() {
    // Apparently windows icons store the color data as BGRA but the last row comes first
    // Load the BGRA icon as the last 128*128 4-bytes chunk of the windows icon
    let mut icon = ui::ICON[..].windows(128*128*4).last().unwrap().to_vec();
    // Reverse it, now it's a forward-ARGB with reversed rows
    icon.reverse();
    // Now we iterate over row chunks of it
    icon.chunks_exact_mut(128*4).for_each(|row| {
        // row is argb but reversed, so now we reverse it
        row.reverse();
        // row is bgra in the correct order, so we iterate over chunks of it and transform the bgra
        // to argb
        row.chunks_exact_mut(4).for_each(|pixel| {
            pixel.reverse();
            // Now we have argb pixels
            pixel.rotate_left(1);
            // And now we have RGBA pixels, nice.
        });
    });

    let settings = Settings {
        default_font: Some(FONT),
        window: iced::window::Settings {
            size: (640, 294),
            icon: iced::window::icon::Icon::from_rgba(icon, 128, 128).ok(),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    };
    Player::run(settings).unwrap();
}
