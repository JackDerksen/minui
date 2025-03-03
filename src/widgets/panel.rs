use super::{Alignment, BorderChars, TextBlock, Widget};
use crate::widgets::common::WindowView;
use crate::{Color, ColorPair, Window};

pub struct Panel {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    header_text: String,
    body_content: PanelContent,
    header_style: BorderChars,
    body_style: BorderChars,
    header_color: Option<ColorPair>,
    body_color: Option<ColorPair>,
    header_border_color: Option<ColorPair>,
    body_border_color: Option<ColorPair>,
    padding: u16,
    alignment: Alignment, // For the body only; header will always be centered
    auto_size: bool,
}

pub(crate) enum PanelContent {
    Text(String),
    Block(Box<TextBlock>),
}

impl Panel {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            header_text: "".to_string(),
            body_content: PanelContent::Text(String::new()),
            header_style: BorderChars::single_line(),
            body_style: BorderChars::single_line(),
            header_color: None,
            body_color: None,
            header_border_color: None,
            body_border_color: None,
            padding: 1,
            alignment: Alignment::Left,
            auto_size: true, // Auto sizes the panel by default
        }
    }

    pub fn with_header(mut self, text: impl Into<String>) -> Self {
        self.header_text = text.into();
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    pub fn with_body(mut self, text: impl Into<String>) -> Self {
        self.body_content = PanelContent::Text(text.into());
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    pub fn with_body_block(mut self, text_block: TextBlock) -> Self {
        self.body_content = PanelContent::Block(Box::new(text_block));
        if self.auto_size {
            self.adjust_size();
        }
        self
    }

    pub fn with_header_style(mut self, style: BorderChars) -> Self {
        self.header_style = style;
        self
    }

    pub fn with_body_style(mut self, style: BorderChars) -> Self {
        self.body_style = style;
        self
    }

    pub fn with_header_color(mut self, color: Option<ColorPair>) -> Self {
        self.header_color = color;
        self
    }

    pub fn with_body_color(mut self, color: Option<ColorPair>) -> Self {
        self.body_color = color;
        self
    }

    pub fn with_header_border_color(mut self, color: Color) -> Self {
        self.header_border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    pub fn with_body_border_color(mut self, color: Color) -> Self {
        self.body_border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn set_header(&mut self, text: impl Into<String>) {
        self.header_text = text.into();
    }

    pub fn set_body(&mut self, text: impl Into<String>) {
        self.body_content = PanelContent::Text(text.into());
    }

    pub fn with_auto_size(mut self, auto_size: bool) -> Self {
        self.auto_size = auto_size;
        self
    }

    fn adjust_size(&mut self) {
        // Calculate required width
        let header_width = self.header_text.len() as u16 + 4;

        let body_width = match &self.body_content {
            PanelContent::Text(text) => {
                let body_lines: Vec<&str> = text.lines().collect();
                body_lines
                    .iter()
                    .map(|line| line.len() as u16)
                    .max()
                    .unwrap_or(0)
            }
            PanelContent::Block(block) => block.get_size().0,
        };

        let max_body_width = body_width + (self.padding * 2) + 2;
        self.width = header_width.max(max_body_width);

        // Calculate required height
        let body_height = match &self.body_content {
            PanelContent::Text(text) => text.lines().count() as u16,
            PanelContent::Block(block) => block.get_size().1,
        };

        self.height = body_height + 5; // 3 lines for the header + body content + bottom border
    }

    fn get_inner_dimensions(&self) -> (u16, u16) {
        let inner_width = self.width.saturating_sub(2);
        let inner_height = self.height.saturating_sub(4);
        (inner_width, inner_height)
    }
}

impl Widget for Panel {
    // Draws the complete panel including header, borders, and content.
    //
    // The drawing process:
    // 1. Draws the header section with its borders
    // 2. Draws the centered header text
    // 3. Draws the separator between header and body
    // 4. Draws the body borders
    // 5. Draws the body content with proper alignment
    fn draw(&self, window: &mut dyn Window) -> crate::Result<()> {
        // Draw header section
        if let Some(color) = self.header_border_color {
            window.write_str_colored(
                self.y,
                self.x,
                &self.header_style.top_left.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y,
                self.x + self.width - 1,
                &self.header_style.top_right.to_string(),
                color,
            )?;

            // Top border
            for i in 1..self.width - 1 {
                window.write_str_colored(
                    self.y,
                    self.x + i,
                    &self.header_style.horizontal.to_string(),
                    color,
                )?;
            }

            // Header vertical borders
            window.write_str_colored(
                self.y + 1,
                self.x,
                &self.header_style.vertical.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y + 1,
                self.x + self.width - 1,
                &self.header_style.vertical.to_string(),
                color,
            )?;
        } else {
            window.write_str(self.y, self.x, &self.header_style.top_left.to_string())?;
            window.write_str(
                self.y,
                self.x + self.width - 1,
                &self.header_style.top_right.to_string(),
            )?;

            // Top border
            for i in 1..self.width - 1 {
                window.write_str(
                    self.y,
                    self.x + i,
                    &self.header_style.horizontal.to_string(),
                )?;
            }

            // Header vertical borders
            window.write_str(self.y + 1, self.x, &self.header_style.vertical.to_string())?;
            window.write_str(
                self.y + 1,
                self.x + self.width - 1,
                &self.header_style.vertical.to_string(),
            )?;
        }

        // Draw centered header text
        if !self.header_text.is_empty() {
            let text_x = self.x + (self.width - self.header_text.len() as u16) / 2;
            if let Some(colors) = self.header_color {
                window.write_str_colored(self.y + 1, text_x, &self.header_text, colors)?;
            } else {
                window.write_str(self.y + 1, text_x, &self.header_text)?;
            }
        }

        // Header bottom border (with side edge intersections)
        if let Some(color) = self.header_border_color {
            window.write_str_colored(
                self.y + 2,
                self.x,
                &self.header_style.intersect_left.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y + 2,
                self.x + self.width - 1,
                &self.header_style.intersect_right.to_string(),
                color,
            )?;
            for i in 1..self.width - 1 {
                window.write_str_colored(
                    self.y + 2,
                    self.x + i,
                    &self.header_style.horizontal.to_string(),
                    color,
                )?;
            }
        } else {
            window.write_str(
                self.y + 2,
                self.x,
                &self.header_style.intersect_left.to_string(),
            )?;
            window.write_str(
                self.y + 2,
                self.x + self.width - 1,
                &self.header_style.intersect_right.to_string(),
            )?;
            for i in 1..self.width - 1 {
                window.write_str(
                    self.y + 2,
                    self.x + i,
                    &self.header_style.horizontal.to_string(),
                )?;
            }
        }

        // Draw body content
        let (inner_width, inner_height) = self.get_inner_dimensions();
        let body_start_y = self.y + 3;

        // Body vertical borders
        if let Some(color) = self.body_border_color {
            for i in 0..inner_height {
                window.write_str_colored(
                    body_start_y + i,
                    self.x,
                    &self.body_style.vertical.to_string(),
                    color,
                )?;
                window.write_str_colored(
                    body_start_y + i,
                    self.x + self.width - 1,
                    &self.body_style.vertical.to_string(),
                    color,
                )?;
            }
        } else {
            for i in 0..inner_height {
                window.write_str(
                    body_start_y + i,
                    self.x,
                    &self.body_style.vertical.to_string(),
                )?;
                window.write_str(
                    body_start_y + i,
                    self.x + self.width - 1,
                    &self.body_style.vertical.to_string(),
                )?;
            }
        }

        // Draw body content based on type
        match &self.body_content {
            PanelContent::Text(text) => {
                let content_x = self.x + 1 + self.padding;
                let content_width = inner_width - (self.padding * 2);

                for (i, line) in text.lines().enumerate() {
                    if (i as u16) >= inner_height {
                        break;
                    }

                    let line_x = match self.alignment {
                        Alignment::Left => content_x,
                        Alignment::Center => content_x + (content_width - line.len() as u16) / 2,
                        Alignment::Right => content_x + content_width - line.len() as u16,
                    };

                    if let Some(colors) = self.body_color {
                        window.write_str_colored(body_start_y + i as u16, line_x, line, colors)?;
                    } else {
                        window.write_str(body_start_y + i as u16, line_x, line)?;
                    }
                }
            }
            PanelContent::Block(block) => {
                let mut view = WindowView {
                    window,
                    x_offset: self.x + 1 + self.padding,
                    y_offset: body_start_y,
                    width: inner_width - (self.padding * 2),
                    height: inner_height,
                };
                block.draw(&mut view)?;
            }
        }

        // Bottom border
        if let Some(color) = self.body_border_color {
            window.write_str_colored(
                self.y + self.height - 1,
                self.x,
                &self.body_style.bottom_left.to_string(),
                color,
            )?;
            window.write_str_colored(
                self.y + self.height - 1,
                self.x + self.width - 1,
                &self.body_style.bottom_right.to_string(),
                color,
            )?;
            for i in 1..self.width - 1 {
                window.write_str_colored(
                    self.y + self.height - 1,
                    self.x + i,
                    &self.body_style.horizontal.to_string(),
                    color,
                )?;
            }
        } else {
            window.write_str(
                self.y + self.height - 1,
                self.x,
                &self.body_style.bottom_left.to_string(),
            )?;
            window.write_str(
                self.y + self.height - 1,
                self.x + self.width - 1,
                &self.body_style.bottom_right.to_string(),
            )?;
            for i in 1..self.width - 1 {
                window.write_str(
                    self.y + self.height - 1,
                    self.x + i,
                    &self.body_style.horizontal.to_string(),
                )?;
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
