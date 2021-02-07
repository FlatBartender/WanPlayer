use iced::*;

pub const PLAY_SVG: &str = include_str!("resources/play.svg");
pub const PAUSE_SVG: &str = include_str!("resources/pause.svg");
pub const NO_IMAGE: &[u8] = include_bytes!("resources/gr-logo-placeholder.png");


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
            background: Some(iced::Background::Color(iced::Color::new(26.0 / 255.0, 21.0 / 255.0, 55.0 / 255.0, 1.0))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: iced::Color::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}
