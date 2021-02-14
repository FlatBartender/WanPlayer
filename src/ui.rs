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

use iced::*;

use super::PlayerMessage;

pub const PLAY_SVG: &str = include_str!("resources/play.svg");
pub const PAUSE_SVG: &str = include_str!("resources/pause.svg");
pub const ICON: &[u8] = include_bytes!("resources/wan_player.ico");
pub const NO_IMAGE: &[u8] = include_bytes!("resources/not_found.png");

pub struct PlayPauseStyle;
impl widget::button::StyleSheet for PlayPauseStyle {
    fn active(&self) -> widget::button::Style {
        widget::button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::new(0.0, 0.0, 0.0, 0.0),
            text_color: Color::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

pub struct VolumeSliderStyle;
impl widget::slider::StyleSheet for VolumeSliderStyle {
    fn active(&self) -> widget::slider::Style {
        widget::slider::Style {
            rail_colors: (
                iced::Color::new(1.0, 1.0, 1.0, 1.0),
                iced::Color::new(0.0, 0.0, 0.0, 0.0),
            ),
            handle: widget::slider::Handle {
                shape: widget::slider::HandleShape::Circle { radius: 8.0 },
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

pub struct SongProgressStyle;
impl widget::progress_bar::StyleSheet for SongProgressStyle {
    fn style(&self) -> widget::progress_bar::Style {
        widget::progress_bar::Style {
            bar: iced::Background::Color(iced::Color::new(15.0 / 255.0, 135.0 / 255.0, 255.0 / 255.0, 1.0)),
            background: iced::Background::Color(iced::Color::new(9.0 / 255.0, 81.0 / 255.0, 153.0 / 255.0, 1.0)),
            border_radius: 0.0,
        }
    }
}

pub struct PlayerStyle;
impl widget::container::StyleSheet for PlayerStyle {
    fn style(&self) -> widget::container::Style {
        widget::container::Style {
            text_color: Some(iced::Color::new(1.0, 1.0, 1.0, 1.0)),
            background: Some(iced::Background::Color(iced::Color::new(
                26.0 / 255.0,
                21.0 / 255.0,
                55.0 / 255.0,
                1.0,
            ))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: iced::Color::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

pub fn album_art_widget(album_image: &Option<Vec<u8>>) -> widget::Image {
    widget::Image::new(widget::image::Handle::from_memory(if let Some(ref art) = album_image {
        art.clone()
    } else {
        NO_IMAGE.to_vec()
    }))
    .height(iced::Length::Units(200))
    .width(iced::Length::Units(200))
}

pub fn elapsed_widget(
    song_info: &Option<super::gensokyo_radio::GRApiAnswer>,
    volume: u8,
) -> widget::Row<PlayerMessage> {
    let elapsed_row = widget::Row::new();
    let elapsed_row = if let Some(ref song_info) = song_info {
        elapsed_row
            .push(
                widget::Text::new(format!(
                    "{}:{:02}",
                    song_info.songtimes.played / 60,
                    song_info.songtimes.played % 60
                ))
                .width(iced::Length::Shrink),
            )
            .push(
                widget::Text::new(format!("{}%", volume))
                    .width(iced::Length::Fill)
                    .horizontal_alignment(iced::HorizontalAlignment::Center),
            )
            .push(
                widget::Text::new(format!(
                    "{}:{:02}",
                    song_info.songtimes.duration / 60,
                    song_info.songtimes.duration % 60
                ))
                .width(iced::Length::Shrink),
            )
    } else {
        elapsed_row
            .push(widget::Text::new("--:--"))
            .push(
                widget::Text::new(format!("{}%", volume))
                    .width(iced::Length::Fill)
                    .horizontal_alignment(iced::HorizontalAlignment::Center),
            )
            .push(widget::Text::new("--:--"))
    };

    elapsed_row
}

pub fn progress_widget(song_info: &Option<super::gensokyo_radio::GRApiAnswer>) -> widget::ProgressBar {
    if let Some(ref song_info) = song_info {
        widget::ProgressBar::new(
            0.0..=song_info.songtimes.duration as f32,
            song_info.songtimes.played as f32,
        )
    } else {
        widget::ProgressBar::new(0.0..=100.0, 0.0)
    }
    .style(SongProgressStyle)
    .height(iced::Length::Units(8))
}
