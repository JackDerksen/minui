//! Slider widget.
//!
//! This widget is designed to be used with MinUI's Phase 1 interaction model:
//! - You draw the slider like any other widget.
//! - You register its absolute `WidgetArea` in an `InteractionCache` during `draw()`.
//! - You route mouse events from your app's update loop into `Slider::handle_event(...)`,
//!   providing the same `WidgetArea` you registered.
//!
//! The slider supports both vertical and horizontal orientations and uses a
//! "virtual half-cell" technique (2x resolution along the main axis) inspired by
//! OpenTUI's slider. This allows smoother thumb movement even with coarse terminal cells.
//!
//! Notes / limitations:
//! - This implementation is intentionally event-helpers-first. The slider itself does not
//!   know its absolute position (widgets generally draw at (0,0) inside a `WindowView`).
//! - Rendering uses Unicode block characters where possible.

use crate::widgets::{Widget, WidgetArea};
use crate::{
    Color, ColorPair, Event, InteractionCache, InteractionId, MouseButton, Result, Window,
};

/// Slider orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SliderOrientation {
    Vertical,
    Horizontal,
}

/// Slider configuration.
#[derive(Debug, Clone, Copy)]
pub struct SliderOptions {
    /// Orientation of the slider track.
    pub orientation: SliderOrientation,

    /// Minimum value.
    pub min: f32,
    /// Maximum value.
    pub max: f32,
    /// Current value.
    pub value: f32,

    /// Size of the viewport (used to compute thumb size).
    ///
    /// For scrollbars, this corresponds to "viewport size" relative to content range.
    /// If you don't care about proportional thumbs, set this to `1.0`.
    pub viewport_size: f32,

    /// Background (track) color.
    pub track_color: ColorPair,
    /// Foreground (thumb) color.
    pub thumb_color: ColorPair,

    /// Minimum thumb size in virtual units (half-cells).
    pub min_thumb_virtual: u16,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            orientation: SliderOrientation::Vertical,
            min: 0.0,
            max: 100.0,
            value: 0.0,
            viewport_size: 1.0,
            track_color: ColorPair::new(Color::DarkGray, Color::Transparent),
            thumb_color: ColorPair::new(Color::White, Color::Transparent),
            min_thumb_virtual: 1,
        }
    }
}

/// A slider widget with rendering + mouse interaction helpers.
///
/// The slider's `get_size()` is explicit; in practice you'll embed this in a `Container` or
/// place it using a known `WindowView` size.
pub struct Slider {
    width: u16,
    height: u16,

    opts: SliderOptions,

    // Drag state (Phase 1 event routing expects the app to call handle_event).
    is_dragging: bool,
    drag_offset_virtual: u16,
}

impl Slider {
    /// Create a new slider with a fixed size.
    pub fn new(width: u16, height: u16, opts: SliderOptions) -> Self {
        Self {
            width,
            height,
            opts,
            is_dragging: false,
            drag_offset_virtual: 0,
        }
    }

    /// Register this slider's interactive area in `ui` under `id`.
    ///
    /// This is optional / opt-in: it does not change `Widget::draw()` and it does not enforce
    /// any routing policy. It simply answers: "what did the user click/drag?"
    ///
    /// `area` must be the absolute area the slider occupies in terminal coordinates.
    ///
    /// Suggested routing:
    /// - register as `draggable` so drags/clicks can be routed into `Slider::handle_event(...)`.
    pub fn register_with_id(&self, ui: &mut InteractionCache, area: WidgetArea, id: InteractionId) {
        ui.register_draggable(id, area);
    }

    /// Convenience constructor: vertical slider with width 1 (or 2) and given height.
    pub fn vertical(height: u16) -> Self {
        Self::new(
            1,
            height,
            SliderOptions {
                orientation: SliderOrientation::Vertical,
                ..SliderOptions::default()
            },
        )
    }

    /// Convenience constructor: horizontal slider with height 1 and given width.
    pub fn horizontal(width: u16) -> Self {
        Self::new(
            width,
            1,
            SliderOptions {
                orientation: SliderOrientation::Horizontal,
                ..SliderOptions::default()
            },
        )
    }

    /// Set size.
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Mutably set size without consuming the slider (preserves drag state).
    pub fn set_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    /// Set colors.
    pub fn with_colors(mut self, track: ColorPair, thumb: ColorPair) -> Self {
        self.opts.track_color = track;
        self.opts.thumb_color = thumb;
        self
    }

    /// Mutably set colors without consuming the slider (preserves drag state).
    pub fn set_colors(&mut self, track: ColorPair, thumb: ColorPair) {
        self.opts.track_color = track;
        self.opts.thumb_color = thumb;
    }

    /// Set value range.
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.opts.min = min;
        self.opts.max = max;
        self.set_value(self.opts.value);
        self
    }

    /// Mutably set min/max range without consuming the slider (preserves drag state).
    pub fn set_range(&mut self, min: f32, max: f32) {
        self.opts.min = min;
        self.opts.max = max;
        self.set_value(self.opts.value);
    }

    /// Set viewport size (affects thumb size).
    pub fn with_viewport_size(mut self, viewport_size: f32) -> Self {
        self.opts.viewport_size = viewport_size;
        self
    }

    /// Get current value.
    pub fn value(&self) -> f32 {
        self.opts.value
    }

    /// Set current value (clamped).
    pub fn set_value(&mut self, value: f32) {
        let (min, max) = self.normalized_range();
        let v = value.clamp(min, max);
        self.opts.value = v;
    }

    /// Get min.
    pub fn min(&self) -> f32 {
        self.opts.min
    }

    /// Get max.
    pub fn max(&self) -> f32 {
        self.opts.max
    }

    // Note: `set_range` is implemented earlier in this impl block.
    // Keeping a single definition avoids duplicate-method compilation errors.

    /// Set viewport size (clamped).
    pub fn set_viewport_size(&mut self, viewport_size: f32) {
        self.opts.viewport_size = viewport_size.max(0.0);
    }

    /// Returns whether a drag is in progress.
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Stop dragging (useful if mouse capture is lost in app-level routing).
    pub fn cancel_drag(&mut self) {
        self.is_dragging = false;
        self.drag_offset_virtual = 0;
    }

    /// Handle an event routed from the app, updating slider value if applicable.
    ///
    /// `area` must be the absolute area the slider occupies (registered during draw).
    ///
    /// Returns `true` if the slider value changed or drag state changed.
    pub fn handle_event(&mut self, event: &Event, area: crate::widgets::WidgetArea) -> bool {
        match *event {
            Event::MouseClick { x, y, button } => {
                if button != MouseButton::Left {
                    return false;
                }
                if !area.contains_point(x, y) {
                    return false;
                }

                // Click inside: jump to position and begin dragging.
                let changed = self.update_value_from_mouse_direct(x, y, area);
                self.is_dragging = true;
                self.drag_offset_virtual = self.calculate_drag_offset_virtual(x, y, area);
                true || changed
            }
            Event::MouseDrag { x, y, button } => {
                if button != MouseButton::Left {
                    return false;
                }
                if !self.is_dragging {
                    return false;
                }
                // During drag we allow out-of-area movement (common UX). We clamp inside.
                self.update_value_from_mouse_with_offset(x, y, area, self.drag_offset_virtual)
            }
            Event::MouseRelease { button, .. } => {
                if button != MouseButton::Left {
                    return false;
                }
                if self.is_dragging {
                    self.is_dragging = false;
                    self.drag_offset_virtual = 0;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    // ---- Rendering ----

    fn normalized_range(&self) -> (f32, f32) {
        if self.opts.max >= self.opts.min {
            (self.opts.min, self.opts.max)
        } else {
            (self.opts.max, self.opts.min)
        }
    }

    fn axis_len_cells(&self) -> u16 {
        match self.opts.orientation {
            SliderOrientation::Vertical => self.height,
            SliderOrientation::Horizontal => self.width,
        }
    }

    fn axis_len_virtual(&self) -> u16 {
        self.axis_len_cells().saturating_mul(2)
    }

    fn range(&self) -> f32 {
        let (min, max) = self.normalized_range();
        (max - min).max(0.0)
    }

    /// Thumb size in virtual units (half-cells) based on viewport sizing.
    fn thumb_size_virtual(&self) -> u16 {
        let track = self.axis_len_virtual();
        if track == 0 {
            return 0;
        }

        let range = self.range();
        if range <= 0.0 {
            return track;
        }

        let viewport = self.opts.viewport_size.max(1.0);
        let content = range + viewport;

        if content <= viewport {
            return track;
        }

        let ratio = viewport / content;
        let size = (track as f32 * ratio).floor() as i32;

        let size = size.max(self.opts.min_thumb_virtual as i32);
        let size = size.min(track as i32);
        size as u16
    }

    /// Thumb start in virtual units (half-cells).
    fn thumb_start_virtual(&self) -> u16 {
        let track = self.axis_len_virtual();
        if track == 0 {
            return 0;
        }

        let range = self.range();
        if range <= 0.0 {
            return 0;
        }

        let (min, _max) = self.normalized_range();
        let value_ratio = ((self.opts.value - min) / range).clamp(0.0, 1.0);

        let thumb = self.thumb_size_virtual();
        let max_start = track.saturating_sub(thumb);

        (value_ratio * max_start as f32).round() as u16
    }

    /// Thumb rect in *cells* in local widget coords.
    ///
    /// Note: currently unused, but intentionally kept for future work:
    /// - hit-testing (e.g. detecting whether a mouse-down landed inside the thumb vs track)
    /// - hover/cursor proximity effects (highlight thumb on hover)
    /// - debug overlays / visualizing thumb geometry during layout tuning
    #[allow(dead_code)]
    fn thumb_rect_local(&self) -> (u16, u16, u16, u16) {
        let thumb_size_v = self.thumb_size_virtual();
        let thumb_start_v = self.thumb_start_virtual();
        let thumb_end_v = thumb_start_v.saturating_add(thumb_size_v);

        let real_start = thumb_start_v / 2;
        // ceil((start+size)/2) - start
        let real_end_exclusive = (thumb_end_v + 1) / 2; // ceil division by 2
        let real_size = real_end_exclusive.saturating_sub(real_start).max(1);

        match self.opts.orientation {
            SliderOrientation::Vertical => (0, real_start, self.width.max(1), real_size),
            SliderOrientation::Horizontal => (real_start, 0, real_size, self.height.max(1)),
        }
    }

    fn draw_vertical(&self, window: &mut dyn Window) -> Result<()> {
        // Track fill
        for y in 0..self.height {
            for x in 0..self.width.max(1) {
                window.write_str_colored(y, x, " ", self.opts.track_color)?;
            }
        }

        // Thumb fill with half-cell nuance:
        // Use '█' full, or '▀/▄' for partial coverage per row in virtual units.
        let track_v = self.axis_len_virtual();
        if track_v == 0 {
            return Ok(());
        }

        let thumb_v = self.thumb_size_virtual();
        let start_v = self.thumb_start_virtual();
        let end_v = start_v.saturating_add(thumb_v);

        let real_start = start_v / 2;
        let real_end = ((end_v + 1) / 2).saturating_sub(1); // inclusive
        if self.height == 0 {
            return Ok(());
        }

        let start_y = real_start.min(self.height.saturating_sub(1));
        let end_y = real_end.min(self.height.saturating_sub(1));

        for real_y in start_y..=end_y {
            let cell_v_start = real_y.saturating_mul(2);
            let cell_v_end = cell_v_start.saturating_add(2);

            let cov_start = start_v.max(cell_v_start);
            let cov_end = end_v.min(cell_v_end);
            let coverage = cov_end.saturating_sub(cov_start);

            let ch = if coverage >= 2 {
                '█'
            } else if coverage == 1 {
                // In vertical, coverage can be top half ('▀') or bottom half ('▄')
                if cov_start == cell_v_start {
                    '▀'
                } else {
                    '▄'
                }
            } else {
                ' '
            };

            if ch != ' ' {
                for x in 0..self.width.max(1) {
                    window.write_str_colored(real_y, x, &ch.to_string(), self.opts.thumb_color)?;
                }
            }
        }

        Ok(())
    }

    fn draw_horizontal(&self, window: &mut dyn Window) -> Result<()> {
        // Track fill
        for y in 0..self.height.max(1) {
            for x in 0..self.width {
                window.write_str_colored(y, x, " ", self.opts.track_color)?;
            }
        }

        let track_v = self.axis_len_virtual();
        if track_v == 0 {
            return Ok(());
        }

        let thumb_v = self.thumb_size_virtual();
        let start_v = self.thumb_start_virtual();
        let end_v = start_v.saturating_add(thumb_v);

        let real_start = start_v / 2;
        let real_end = ((end_v + 1) / 2).saturating_sub(1); // inclusive
        if self.width == 0 {
            return Ok(());
        }

        let start_x = real_start.min(self.width.saturating_sub(1));
        let end_x = real_end.min(self.width.saturating_sub(1));

        for real_x in start_x..=end_x {
            let cell_v_start = real_x.saturating_mul(2);
            let cell_v_end = cell_v_start.saturating_add(2);

            let cov_start = start_v.max(cell_v_start);
            let cov_end = end_v.min(cell_v_end);
            let coverage = cov_end.saturating_sub(cov_start);

            let ch = if coverage >= 2 {
                '█'
            } else if coverage == 1 {
                // In horizontal, half coverage uses left/right half blocks.
                if cov_start == cell_v_start {
                    '▌'
                } else {
                    '▐'
                }
            } else {
                ' '
            };

            if ch != ' ' {
                for y in 0..self.height.max(1) {
                    window.write_str_colored(y, real_x, &ch.to_string(), self.opts.thumb_color)?;
                }
            }
        }

        Ok(())
    }

    // ---- Mouse mapping helpers ----

    fn clamp_mouse_to_area(&self, x: u16, y: u16, area: crate::widgets::WidgetArea) -> (u16, u16) {
        let cx = x.clamp(area.x, area.right().saturating_sub(1));
        let cy = y.clamp(area.y, area.bottom().saturating_sub(1));
        (cx, cy)
    }

    fn calculate_drag_offset_virtual(
        &self,
        x: u16,
        y: u16,
        area: crate::widgets::WidgetArea,
    ) -> u16 {
        let (cx, cy) = self.clamp_mouse_to_area(x, y, area);

        // Local cell pos along main axis, doubled into virtual units.
        let mouse_axis_cells = match self.opts.orientation {
            SliderOrientation::Vertical => cy.saturating_sub(area.y),
            SliderOrientation::Horizontal => cx.saturating_sub(area.x),
        };
        let mouse_axis_v = mouse_axis_cells.saturating_mul(2);

        let thumb_start_v = self.thumb_start_virtual();
        let thumb_size_v = self.thumb_size_virtual();

        // Offset within thumb so dragging preserves relative grab point.
        let offset = mouse_axis_v.saturating_sub(thumb_start_v);
        offset.min(thumb_size_v)
    }

    fn update_value_from_mouse_direct(
        &mut self,
        x: u16,
        y: u16,
        area: crate::widgets::WidgetArea,
    ) -> bool {
        let old = self.opts.value;

        let (cx, cy) = self.clamp_mouse_to_area(x, y, area);

        let track_cells = self.axis_len_cells().max(1);
        let mouse_cells = match self.opts.orientation {
            SliderOrientation::Vertical => cy.saturating_sub(area.y),
            SliderOrientation::Horizontal => cx.saturating_sub(area.x),
        };

        let ratio = (mouse_cells as f32 / track_cells as f32).clamp(0.0, 1.0);
        let (min, _max) = self.normalized_range();
        let value = min + ratio * self.range();

        self.set_value(value);
        (self.opts.value - old).abs() > f32::EPSILON
    }

    fn update_value_from_mouse_with_offset(
        &mut self,
        x: u16,
        y: u16,
        area: crate::widgets::WidgetArea,
        offset_virtual: u16,
    ) -> bool {
        let old = self.opts.value;

        let (cx, cy) = self.clamp_mouse_to_area(x, y, area);

        let track_cells = self.axis_len_cells().max(1);
        let track_v = track_cells.saturating_mul(2);

        let mouse_cells = match self.opts.orientation {
            SliderOrientation::Vertical => cy.saturating_sub(area.y),
            SliderOrientation::Horizontal => cx.saturating_sub(area.x),
        };
        let mouse_v = mouse_cells.saturating_mul(2);

        let thumb_v = self.thumb_size_virtual().max(1);
        let max_start = track_v.saturating_sub(thumb_v);

        let mut desired_start = mouse_v.saturating_sub(offset_virtual);
        desired_start = desired_start.min(max_start);

        let ratio = if max_start == 0 {
            0.0
        } else {
            (desired_start as f32 / max_start as f32).clamp(0.0, 1.0)
        };

        let (min, _max) = self.normalized_range();
        let value = min + ratio * self.range();

        self.set_value(value);
        (self.opts.value - old).abs() > f32::EPSILON
    }
}

impl Widget for Slider {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        match self.opts.orientation {
            SliderOrientation::Vertical => self.draw_vertical(window),
            SliderOrientation::Horizontal => self.draw_horizontal(window),
        }
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0)
    }
}
