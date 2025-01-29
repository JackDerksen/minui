use crate::{Window, Result, ColorPair};
use super::{Alignment, Widget};

#[derive(Debug, Clone, Copy)]
pub enum TextWrapMode {
    None,           // No wrapping, clip text
    Wrap,           // Wrap at widget boundary
    WrapWords,      // Wrap at word boundaries
}

#[derive(Debug, Clone, Copy)]
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
}

pub struct TextBlock {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    text: String,
    colors: Option<ColorPair>,
    wrap_mode: TextWrapMode,
    h_align: Alignment,  // Reuse Label's Alignment enum
    v_align: VerticalAlignment,
    scroll_offset: u16,  // For scrollable text
}

impl TextBlock {
    pub fn new(x: u16, y: u16, width: u16, height: u16, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            width,
            height,
            text: text.into(),
            colors: None,
            wrap_mode: TextWrapMode::Wrap,
            h_align: Alignment::Left,
            v_align: VerticalAlignment::Top,
            scroll_offset: 0,
        }
    }

    pub fn with_colors(mut self, colors: ColorPair) -> Self {
        self.colors = Some(colors);
        self
    }

    pub fn with_wrap_mode(mut self, mode: TextWrapMode) -> Self {
        self.wrap_mode = mode;
        self
    }

    pub fn with_alignment(mut self, h_align: Alignment, v_align: VerticalAlignment) -> Self {
        self.h_align = h_align;
        self.v_align = v_align;
        self
    }

    fn get_wrapped_lines(&self) -> Vec<String> {
        match self.wrap_mode {
            TextWrapMode::None => self.text.lines().map(String::from).collect(),
            TextWrapMode::Wrap => {
                // Simple character-based wrapping
                self.text
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(self.width as usize)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect()
            }
            TextWrapMode::WrapWords => {
                let mut lines = Vec::new();
                let mut current_line = String::new();

                for word in self.text.split_whitespace() {
                    if current_line.len() + word.len() + 1 <= self.width as usize {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    } else {
                        if !current_line.is_empty() {
                            lines.push(current_line);
                        }
                        current_line = word.to_string();
                    }
                }

                if !current_line.is_empty() {
                    lines.push(current_line);
                }

                lines
            }
        }
    }

    pub fn scroll_to(&mut self, line: u16) {
        self.scroll_offset = line;
    }

    pub fn scroll_by(&mut self, delta: i16) {
        self.scroll_offset = if delta.is_negative() {
            self.scroll_offset.saturating_sub(delta.abs() as u16)
        } else {
            self.scroll_offset.saturating_add(delta as u16)
        };
    }
}

impl Widget for TextBlock {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        let lines = self.get_wrapped_lines();
        let (window_width, window_height) = window.get_size();

        // Calculate block position horizontally
        let block_x = match self.h_align {
            Alignment::Left => self.x,
            Alignment::Center => {
                self.x + (window_width.saturating_sub(self.width)) / 2
            }
            Alignment::Right => {
                self.x + window_width.saturating_sub(self.width)
            }
        };

        // Calculate block position vertically
        let total_lines = lines.len();
        let visible_lines = total_lines.min(self.height as usize);
        let block_y = match self.v_align {
            VerticalAlignment::Top => self.y,
            VerticalAlignment::Middle => {
                self.y + (window_height.saturating_sub(visible_lines as u16)) / 2
            }
            VerticalAlignment::Bottom => {
                self.y + window_height.saturating_sub(visible_lines as u16)
            }
        };

        // Get displayable lines starting from the calculated vertical position
        let start_line = self.scroll_offset as usize;
        let display_lines: Vec<String> = lines
            .into_iter()
            .skip(start_line)
            .take(self.height as usize)
            .collect();

        // Draw each line
        for (i, line) in display_lines.iter().enumerate() {
            let line_y = block_y + i as u16;
            if line_y >= block_y + self.height {
                break;
            }

            // Calculate x position for each line relative to the block's position
            let line_x = match self.h_align {
                Alignment::Left => block_x,
                Alignment::Center => {
                    block_x + (self.width.saturating_sub(line.len() as u16)) / 2
                }
                Alignment::Right => {
                    block_x + self.width.saturating_sub(line.len() as u16)
                }
            };

            // Draw the line
            if let Some(colors) = self.colors {
                window.write_str_colored(line_y, line_x, line, colors)?;
            } else {
                window.write_str(line_y, line_x, line)?;
            }
        }

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}