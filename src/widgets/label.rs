use crate::{Window, Result, ColorPair, Color};
use super::Widget;

pub struct Label {
    x: u16,
    y: u16,
    text: String,
    colors: Option<ColorPair>,
    alignment: Alignment,
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Label {
    pub fn new(x: u16, y: u16, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            text: text.into(),
            colors: None,
            alignment: Alignment::Left,
        }
    }

    pub fn with_color(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    pub fn with_text_color(mut self, color: Color) -> Self {
        self.colors = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn get_length(&self) -> u16 {
        self.text.len() as u16
    }

    fn calculate_aligned_x(&self, available_width: Option<u16>) -> u16 {
        if let Some(width) = available_width {
            let text_length = self.get_length();
            match self.alignment {
                Alignment::Left => self.x,
                Alignment::Center => {
                    if text_length < width {
                        self.x + (width - text_length) / 2
                    } else {
                        self.x
                    }
                }
                Alignment::Right => {
                    if text_length < width {
                        self.x + width - text_length
                    } else {
                        self.x
                    }
                }
            }
        } else {
            self.x
        }
    }
}

impl Widget for Label {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // Get window size to calculate available width
        let (window_width, _) = window.get_size();
        let x_pos = self.calculate_aligned_x(Some(window_width));

        match self.colors {
            Some(colors) => window.write_str_colored(self.y, x_pos, &self.text, colors),
            None => window.write_str(self.y, x_pos, &self.text),
        }
    }

    fn get_size(&self) -> (u16, u16) {
        // Width is the text length, height is always 1 for a simple/short label
        (self.text.chars().count() as u16, 1)
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}