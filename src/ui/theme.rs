use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Sangue,
    Ambar,
    Fosforo,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::Sangue, Theme::Ambar, Theme::Fosforo];

    pub fn label(&self) -> &'static str {
        match self {
            Theme::Sangue => "SANGUE",
            Theme::Ambar => "ÂMBAR",
            Theme::Fosforo => "FÓSFORO",
        }
    }

    pub fn palette(&self) -> Palette {
        match self {
            Theme::Sangue => Palette {
                accent: Color::Rgb(255, 42, 42),
                bg: Color::Rgb(7, 4, 6),
                on_accent: Color::Rgb(7, 4, 6),
                peak: (255, 42, 42),
            },
            Theme::Ambar => Palette {
                accent: Color::Rgb(255, 176, 0),
                bg: Color::Rgb(7, 4, 6),
                on_accent: Color::Rgb(7, 4, 6),
                peak: (255, 176, 0),
            },
            Theme::Fosforo => Palette {
                accent: Color::Rgb(61, 255, 116),
                bg: Color::Rgb(7, 4, 6),
                on_accent: Color::Rgb(7, 4, 6),
                peak: (61, 255, 116),
            },
        }
    }
}

pub struct Palette {
    accent: Color,
    bg: Color,
    on_accent: Color,
    peak: (u8, u8, u8),
}

impl Palette {
    pub fn glow(&self, intensity: u8) -> Color {
        let (r, g, b) = self.peak;

        let scale = |c: u8| (c as u16 * intensity as u16 / 255) as u8;

        Color::Rgb(scale(r), scale(g), scale(b))
    }
}

#[cfg(test)]
mod tests {

    use ratatui::style::Color;

    use super::Theme;

    #[test]
    fn glow_at_full_intensity_for_sangue_theme() {
        let intensity = 255;

        let sangue_theme = Theme::Sangue;
        let sangue_palette = sangue_theme.palette();

        assert_eq!(sangue_palette.glow(intensity), Color::Rgb(255, 42, 42));
    }

    #[test]
    fn glow_at_full_intensity_for_ambar_theme() {
        let intensity = 255;

        let ambar_theme = Theme::Ambar;
        let ambar_palette = ambar_theme.palette();

        assert_eq!(ambar_palette.glow(intensity), Color::Rgb(255, 176, 0));
    }

    #[test]
    fn glow_at_full_intensity_for_fosforo_theme() {
        let intensity = 255;

        let fosforo_theme = Theme::Fosforo;
        let fosforo_palette = fosforo_theme.palette();

        assert_eq!(fosforo_palette.glow(intensity), Color::Rgb(61, 255, 116));
    }

    #[test]
    fn glow_at_mid_intensity_for_sangue_theme() {
        let intensity = 128;

        let sangue_theme = Theme::Sangue;
        let sangue_palette = sangue_theme.palette();

        assert_eq!(sangue_palette.glow(intensity), Color::Rgb(128, 21, 21));
    }

    #[test]
    fn glow_at_mid_intensity_for_ambar_theme() {
        let intensity = 128;

        let ambar_theme = Theme::Ambar;
        let ambar_palette = ambar_theme.palette();

        assert_eq!(ambar_palette.glow(intensity), Color::Rgb(128, 88, 0));
    }

    #[test]
    fn glow_at_mid_intensity_for_fosforo_theme() {
        let intensity = 128;

        let fosforo_theme = Theme::Fosforo;
        let fosforo_palette = fosforo_theme.palette();

        assert_eq!(fosforo_palette.glow(intensity), Color::Rgb(30, 128, 58));
    }

    #[test]
    fn glow_at_zero_intensity_is_black_for_every_theme() {
        for theme in Theme::ALL {
            assert_eq!(
                theme.palette().glow(0),
                Color::Rgb(0, 0, 0),
                "{} must fade all the way to black",
                theme.label()
            );
        }
    }

    #[test]
    fn every_theme_shares_the_same_background() {
        let reference_bg = Theme::Sangue.palette().bg;

        for theme in Theme::ALL {
            assert_eq!(theme.palette().bg, reference_bg, "{}", theme.label());
        }
    }

    #[test]
    fn the_three_accents_are_all_distinct() {
        let sangue = Theme::Sangue.palette().accent;
        let ambar = Theme::Ambar.palette().accent;
        let fosforo = Theme::Fosforo.palette().accent;

        assert_ne!(sangue, ambar);
        assert_ne!(ambar, fosforo);
        assert_ne!(fosforo, sangue);
    }

    #[test]
    fn text_on_an_accent_chip_is_the_background_showing_through() {
        for theme in Theme::ALL {
            let palette = theme.palette();

            assert_eq!(palette.on_accent, palette.bg, "{}", theme.label());
        }
    }
}
