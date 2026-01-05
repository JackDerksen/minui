//! ScrollBar widget.
//!
//! This widget is designed to work with MinUI's Phase 1 interaction model:
//! - You draw the scrollbar like any other widget.
//! - You register its absolute `WidgetArea` in an `InteractionCache` during `draw()`.
//! - You route mouse/key events from your app's update loop into `ScrollBar::handle_event(...)`,
//!   providing the same `WidgetArea` you registered.
//!
//! The scrollbar binds to a shared `ScrollState` (typically `Rc<RefCell<ScrollState>>`) so it
//! stays in sync with one or more viewports.
//!
//! This implementation is inspired by OpenTUI's ScrollBar/Slider composition, but adapted to
//! MinUI's current draw-only widget trait.
//!
//! ## Pre-draw sync (recommended)
//! `Widget::draw` takes `&self`, so this type cannot update its internal slider/arrows during
//! drawing. To avoid rebuilding a render-only slider every frame, call
//! `ScrollBar::sync_from_state_and_resize_parts()` from your app's update or draw code *before*
//! drawing the scrollbar. When the scrollbar is "synced", it will render using its internal
//! `slider`/arrow widgets, preserving drag state.
//!
//! Notes / limitations:
//! - Arrow buttons are supported for mouse clicks (and basic keyboard keys if routed).
//! - Continuous press-repeat (hold-to-scroll) is intentionally out of scope for Phase 1.
//! - The scrollbar does not perform its own hit-testing; you must pass in the absolute area.
//! - Rendering is intentionally simple and uses block/arrow Unicode characters.

use crate::widgets::controls::slider::{Slider, SliderOptions, SliderOrientation};
use crate::widgets::scroll::{ScrollOrientation, ScrollState};
use crate::widgets::{Widget, WidgetArea, WindowView};
use crate::{
    Color, ColorPair, Event, InteractionCache, InteractionId, MouseButton, Result, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Scroll units used by `scroll_by`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollUnit {
    /// Raw cell units (absolute delta).
    Absolute,
    /// A fraction of viewport size (delta is interpreted as a fraction, see `scroll_by_fraction`).
    Viewport,
    /// A fraction of content size (delta is interpreted as a fraction, see `scroll_by_fraction`).
    Content,
    /// A configurable "step" size (cells) if provided, else 1.
    Step,
}

/// Direction for arrow buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

/// A tiny render-only arrow button widget.
pub struct ArrowButton {
    width: u16,
    height: u16,
    dir: ArrowDirection,
    color: ColorPair,
}

impl ArrowButton {
    pub fn new(dir: ArrowDirection) -> Self {
        Self {
            width: 1,
            height: 1,
            dir,
            color: ColorPair::new(Color::White, Color::Transparent),
        }
    }

    pub fn with_color(mut self, color: ColorPair) -> Self {
        self.color = color;
        self
    }

    /// Mutably set the arrow color without consuming the button.
    ///
    /// This is used by `ScrollBar` mutating setters to avoid moving `ArrowButton`.
    pub fn set_color(&mut self, color: ColorPair) {
        self.color = color;
    }

    pub fn glyph(&self) -> &'static str {
        match self.dir {
            ArrowDirection::Up => "▲",
            ArrowDirection::Down => "▼",
            ArrowDirection::Left => "◀",
            ArrowDirection::Right => "▶",
        }
    }
}

impl Widget for ArrowButton {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // Center in available space if larger than 1x1.
        let x = self.width.saturating_sub(1) / 2;
        let y = self.height.saturating_sub(1) / 2;
        window.write_str_colored(y, x, self.glyph(), self.color)?;
        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0)
    }
}

/// Options for a scrollbar.
#[derive(Debug, Clone)]
pub struct ScrollBarOptions {
    pub orientation: ScrollOrientation,
    pub show_arrows: bool,

    /// Track/slider colors.
    pub track_color: ColorPair,
    pub thumb_color: ColorPair,

    /// Arrow button color.
    pub arrow_color: ColorPair,

    /// Optional step size in cells for arrow clicks / step scrolling.
    pub scroll_step: Option<u16>,
}

impl Default for ScrollBarOptions {
    fn default() -> Self {
        Self {
            orientation: ScrollOrientation::Vertical,
            show_arrows: false,
            track_color: ColorPair::new(Color::DarkGray, Color::Transparent),
            thumb_color: ColorPair::new(Color::White, Color::Transparent),
            arrow_color: ColorPair::new(Color::White, Color::Transparent),
            scroll_step: None,
        }
    }
}

/// A scrollbar widget that binds to a shared `ScrollState` and uses an internal `Slider` for dragging.
pub struct ScrollBar {
    width: u16,
    height: u16,

    opts: ScrollBarOptions,
    state: Rc<RefCell<ScrollState>>,

    // Internal render parts (kept in sync via `sync_from_state_and_resize_parts`).
    slider: Slider,
    start_arrow: ArrowButton,
    end_arrow: ArrowButton,

    /// Whether internal render parts are currently in sync with `state` and size/options.
    ///
    /// Call `sync_from_state_and_resize_parts()` before drawing to keep this true.
    synced: bool,
}

impl ScrollBar {
    /// Create a new scrollbar with fixed size.
    pub fn new(
        width: u16,
        height: u16,
        state: Rc<RefCell<ScrollState>>,
        opts: ScrollBarOptions,
    ) -> Self {
        let (start_dir, end_dir, slider_orientation) = match opts.orientation {
            ScrollOrientation::Vertical => (
                ArrowDirection::Up,
                ArrowDirection::Down,
                SliderOrientation::Vertical,
            ),
            ScrollOrientation::Horizontal => (
                ArrowDirection::Left,
                ArrowDirection::Right,
                SliderOrientation::Horizontal,
            ),
        };

        let mut slider = Slider::new(
            width,
            height,
            SliderOptions {
                orientation: slider_orientation,
                min: 0.0,
                max: 0.0,
                value: 0.0,
                viewport_size: 1.0,
                track_color: opts.track_color,
                thumb_color: opts.thumb_color,
                min_thumb_virtual: 1,
            },
        );

        // Size will be corrected in `sync_from_state_and_resize_parts`.
        slider = slider.with_colors(opts.track_color, opts.thumb_color);

        let mut this = Self {
            width,
            height,
            opts: opts.clone(),
            state,
            slider,
            start_arrow: ArrowButton::new(start_dir).with_color(opts.arrow_color),
            end_arrow: ArrowButton::new(end_dir).with_color(opts.arrow_color),
            synced: false,
        };

        this.sync_from_state_and_resize_parts();
        this
    }

    /// Returns whether the scrollbar thumb is currently being dragged.
    ///
    /// This is primarily intended for app-level policies like auto-hide/show behavior.
    pub fn is_dragging(&self) -> bool {
        self.slider.is_dragging()
    }

    /// Register the scrollbar's interactive regions in `ui` under the provided ids.
    ///
    /// This is optional / opt-in: it does not change `Widget::draw()` and it does not enforce
    /// any routing policy. It simply answers: "what did the user click/drag?"
    ///
    /// Suggested routing:
    /// - Register `root_id` as `focusable` (click target; avoids stealing wheel routing).
    /// - Register `thumb_id` as `draggable` (drag thumb/track).
    /// - If arrows are enabled, register `start_arrow_id` / `end_arrow_id` as `focusable`
    ///   (clickable).
    ///
    /// Notes:
    /// - `area` must be the absolute area the scrollbar occupies in terminal coordinates.
    /// - The helper intentionally does not call `begin_frame()`; the app owns the frame lifecycle.
    pub fn register_with_ids(
        &mut self,
        ui: &mut InteractionCache,
        area: WidgetArea,
        root_id: InteractionId,
        thumb_id: InteractionId,
        start_arrow_id: Option<InteractionId>,
        end_arrow_id: Option<InteractionId>,
    ) {
        // Keep internal slider configured from latest scroll state so computed regions are correct.
        self.sync_from_state_and_resize_parts();

        // Whole scrollbar: treat as a clickable/focusable region (but not a wheel target).
        ui.register_focusable(root_id, area);

        // Thumb/track region: draggable (slider handles click+drag behavior).
        let slider_area = self.slider_area_within(area);
        ui.register_draggable(thumb_id, slider_area);

        // Optional arrow buttons: clickable (focusable is the closest existing flag).
        if self.opts.show_arrows {
            if let Some(id) = start_arrow_id {
                ui.register_focusable(id, self.start_arrow_area_within(area));
            }
            if let Some(id) = end_arrow_id {
                ui.register_focusable(id, self.end_arrow_area_within(area));
            }
        }
    }

    /// Convenience: register just the primary interactive regions (whole bar + thumb/track).
    ///
    /// If you don't care about per-arrow hit-testing, use this.
    pub fn register_with_id(
        &mut self,
        ui: &mut InteractionCache,
        area: WidgetArea,
        root_id: InteractionId,
        thumb_id: InteractionId,
    ) {
        self.register_with_ids(ui, area, root_id, thumb_id, None, None);
    }

    /// Convenience: vertical scrollbar.
    pub fn vertical(height: u16, state: Rc<RefCell<ScrollState>>) -> Self {
        Self::new(
            1,
            height,
            state,
            ScrollBarOptions {
                orientation: ScrollOrientation::Vertical,
                ..ScrollBarOptions::default()
            },
        )
    }

    /// Convenience: horizontal scrollbar.
    pub fn horizontal(width: u16, state: Rc<RefCell<ScrollState>>) -> Self {
        Self::new(
            width,
            1,
            state,
            ScrollBarOptions {
                orientation: ScrollOrientation::Horizontal,
                ..ScrollBarOptions::default()
            },
        )
    }

    /// Set size (and re-sync internal parts).
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self.sync_from_state_and_resize_parts();
        self
    }

    /// Mutably set size without recreating the scrollbar (preserves drag state).
    ///
    /// Call this from your draw/layout code when terminal size changes.
    pub fn set_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.sync_from_state_and_resize_parts();
    }

    pub fn with_show_arrows(mut self, show: bool) -> Self {
        self.opts.show_arrows = show;
        self.sync_from_state_and_resize_parts();
        self
    }

    /// Mutably set whether arrows are shown (preserves drag state).
    pub fn set_show_arrows(&mut self, show: bool) {
        self.opts.show_arrows = show;
        self.sync_from_state_and_resize_parts();
    }

    pub fn with_colors(mut self, track: ColorPair, thumb: ColorPair) -> Self {
        self.opts.track_color = track;
        self.opts.thumb_color = thumb;
        // Avoid moving the slider; keep drag state stable for retained instances.
        self.slider.set_colors(track, thumb);
        self.synced = false;
        self
    }

    /// Mutably set track/thumb colors (preserves drag state).
    pub fn set_colors(&mut self, track: ColorPair, thumb: ColorPair) {
        self.opts.track_color = track;
        self.opts.thumb_color = thumb;
        self.slider.set_colors(track, thumb);
        self.synced = false;
    }

    pub fn with_arrow_color(mut self, color: ColorPair) -> Self {
        self.opts.arrow_color = color;
        // Avoid moving arrow buttons; keep internal parts stable for retained instances.
        self.start_arrow.set_color(color);
        self.end_arrow.set_color(color);
        self.synced = false;
        self
    }

    /// Mutably set arrow color (preserves drag state).
    pub fn set_arrow_color(&mut self, color: ColorPair) {
        self.opts.arrow_color = color;
        self.start_arrow.set_color(color);
        self.end_arrow.set_color(color);
        self.synced = false;
    }

    pub fn with_scroll_step(mut self, step: Option<u16>) -> Self {
        self.opts.scroll_step = step;
        self.synced = false;
        self
    }

    /// Mutably set the scroll step (in cells) without recreating the scrollbar.
    ///
    /// Note: call `sync_from_state_and_resize_parts()` before drawing if you need the
    /// scrollbar to render using its internal slider/arrows (preserving drag state).
    pub fn set_scroll_step(&mut self, step: Option<u16>) {
        self.opts.scroll_step = step;
        self.synced = false;
    }

    /// Access the bound state.
    pub fn scroll_state(&self) -> Rc<RefCell<ScrollState>> {
        self.state.clone()
    }

    /// Returns the scrollbar's main-axis usable track length in cells, accounting for arrow buttons.
    ///
    /// Note: currently unused, but kept as a future hook for:
    /// - click-on-track “page” scrolling behavior (jump by viewport)
    /// - hover/cursor proximity logic for auto-hide/show
    /// - thumb geometry debugging
    #[allow(dead_code)]
    fn track_len_cells(&self) -> u16 {
        let axis = match self.opts.orientation {
            ScrollOrientation::Vertical => self.height,
            ScrollOrientation::Horizontal => self.width,
        };
        if self.opts.show_arrows {
            axis.saturating_sub(2) // 1 cell for start, 1 for end
        } else {
            axis
        }
    }

    /// Returns the absolute `WidgetArea` for the slider track inside the scrollbar.
    fn slider_area_within(&self, outer: WidgetArea) -> WidgetArea {
        match self.opts.orientation {
            ScrollOrientation::Vertical => {
                let top = if self.opts.show_arrows { 1 } else { 0 };
                let h = outer
                    .height
                    .saturating_sub(if self.opts.show_arrows { 2 } else { 0 });
                WidgetArea::new(outer.x, outer.y + top, outer.width, h)
            }
            ScrollOrientation::Horizontal => {
                let left = if self.opts.show_arrows { 1 } else { 0 };
                let w = outer
                    .width
                    .saturating_sub(if self.opts.show_arrows { 2 } else { 0 });
                WidgetArea::new(outer.x + left, outer.y, w, outer.height)
            }
        }
    }

    fn start_arrow_area_within(&self, outer: WidgetArea) -> WidgetArea {
        match self.opts.orientation {
            ScrollOrientation::Vertical => WidgetArea::new(outer.x, outer.y, outer.width, 1),
            ScrollOrientation::Horizontal => WidgetArea::new(outer.x, outer.y, 1, outer.height),
        }
    }

    fn end_arrow_area_within(&self, outer: WidgetArea) -> WidgetArea {
        match self.opts.orientation {
            ScrollOrientation::Vertical => {
                let y = outer.y + outer.height.saturating_sub(1);
                WidgetArea::new(outer.x, y, outer.width, 1)
            }
            ScrollOrientation::Horizontal => {
                let x = outer.x + outer.width.saturating_sub(1);
                WidgetArea::new(x, outer.y, 1, outer.height)
            }
        }
    }

    /// Synchronize slider range/value/viewport from ScrollState and resize internal components.
    ///
    /// This is intended to be called from app code *before* drawing, so the scrollbar can render
    /// using its internal slider/arrows without rebuilding a render-only slider each frame.
    pub fn sync_from_state_and_resize_parts(&mut self) {
        // Configure arrow sizes (always 1 cell along main axis, stretch on cross axis).
        match self.opts.orientation {
            ScrollOrientation::Vertical => {
                self.start_arrow.width = self.width.max(1);
                self.start_arrow.height = 1;
                self.end_arrow.width = self.width.max(1);
                self.end_arrow.height = 1;
            }
            ScrollOrientation::Horizontal => {
                self.start_arrow.width = 1;
                self.start_arrow.height = self.height.max(1);
                self.end_arrow.width = 1;
                self.end_arrow.height = self.height.max(1);
            }
        }

        // Slider occupies the track area.
        let slider_w;
        let slider_h;
        match self.opts.orientation {
            ScrollOrientation::Vertical => {
                slider_w = self.width.max(1);
                slider_h = self
                    .height
                    .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                    .max(1);
            }
            ScrollOrientation::Horizontal => {
                slider_w = self
                    .width
                    .saturating_sub(if self.opts.show_arrows { 2 } else { 0 });
                slider_h = self.height.max(1);
            }
        }

        // Keep at least 1 cell for slider if possible.
        let slider_w = slider_w.max(1);
        let slider_h = slider_h.max(1);

        let s = *self.state.borrow();

        // Scroll range in cells along axis.
        let max_offset = s.max_offset_for(self.opts.orientation);
        let viewport_size = match self.opts.orientation {
            ScrollOrientation::Vertical => s.viewport_size().height,
            ScrollOrientation::Horizontal => s.viewport_size().width,
        };
        let offset = s.offset_for(self.opts.orientation);

        // Update slider in-place to preserve drag state.
        self.slider.set_size(slider_w, slider_h);
        self.slider.set_range(0.0, max_offset as f32);
        self.slider.set_viewport_size(viewport_size.max(1) as f32);
        self.slider.set_value(offset as f32);

        self.synced = true;
    }

    /// Apply slider's current value to the scroll state.
    fn sync_state_from_slider(&mut self) {
        let value = self.slider.value().round().max(0.0);
        let value = value as u16;

        self.state
            .borrow_mut()
            .set_offset_for(self.opts.orientation, value);

        // Ensure slider range/value still consistent after clamping.
        self.sync_from_state_and_resize_parts();
    }

    /// Scroll by an absolute cell delta (signed).
    pub fn scroll_by_cells(&mut self, delta: i16) {
        self.state
            .borrow_mut()
            .scroll_by(self.opts.orientation, delta);
        self.sync_from_state_and_resize_parts();
    }

    /// Scroll by a fraction of viewport/content size (delta_fraction in [-1.0..1.0] typical).
    pub fn scroll_by_fraction(&mut self, delta_fraction: f32, unit: ScrollUnit) {
        let s = *self.state.borrow();
        let multiplier: u16 = match unit {
            ScrollUnit::Absolute => 1,
            ScrollUnit::Viewport => match self.opts.orientation {
                ScrollOrientation::Vertical => s.viewport_size().height,
                ScrollOrientation::Horizontal => s.viewport_size().width,
            },
            ScrollUnit::Content => match self.opts.orientation {
                ScrollOrientation::Vertical => s.content_size().height,
                ScrollOrientation::Horizontal => s.content_size().width,
            },
            ScrollUnit::Step => self.opts.scroll_step.unwrap_or(1),
        }
        .max(1);

        let delta = (multiplier as f32 * delta_fraction).round() as i16;
        self.scroll_by_cells(delta);
    }

    /// Handle an event routed from the app, updating scroll state if applicable.
    ///
    /// `area` must be the absolute area the scrollbar occupies (registered during draw).
    ///
    /// Returns `true` if the scroll offset changed or internal drag state changed.
    pub fn handle_event(&mut self, event: &Event, area: WidgetArea) -> bool {
        // Keep internal slider configured from latest scroll state.
        //
        // This is important for dragging: we want to preserve the internal slider's drag state
        // while still updating its range/value/size from shared scroll state.
        self.sync_from_state_and_resize_parts();

        // Arrow buttons (mouse click)
        if self.opts.show_arrows {
            match *event {
                Event::MouseClick {
                    x,
                    y,
                    button: MouseButton::Left,
                } => {
                    let start_area = self.start_arrow_area_within(area);
                    if start_area.contains_point(x, y) {
                        let step = self.opts.scroll_step.unwrap_or(1) as i16;
                        self.scroll_by_cells(-step);
                        return true;
                    }
                    let end_area = self.end_arrow_area_within(area);
                    if end_area.contains_point(x, y) {
                        let step = self.opts.scroll_step.unwrap_or(1) as i16;
                        self.scroll_by_cells(step);
                        return true;
                    }
                }
                _ => {}
            }
        }

        // Slider area (dragging)
        let slider_area = self.slider_area_within(area);
        let slider_changed = self.slider.handle_event(event, slider_area);

        if slider_changed {
            self.sync_state_from_slider();
            return true;
        }

        // Optional keyboard support (only if routed by app when focused):
        // This is minimal and mirrors OpenTUI-ish behavior.
        match *event {
            Event::KeyUp | Event::Character('k') => {
                if self.opts.orientation == ScrollOrientation::Vertical {
                    self.scroll_by_fraction(-1.0 / 5.0, ScrollUnit::Viewport);
                    return true;
                }
            }
            Event::KeyDown | Event::Character('j') => {
                if self.opts.orientation == ScrollOrientation::Vertical {
                    self.scroll_by_fraction(1.0 / 5.0, ScrollUnit::Viewport);
                    return true;
                }
            }
            Event::KeyLeft | Event::Character('h') => {
                if self.opts.orientation == ScrollOrientation::Horizontal {
                    self.scroll_by_fraction(-1.0 / 5.0, ScrollUnit::Viewport);
                    return true;
                }
            }
            Event::KeyRight | Event::Character('l') => {
                if self.opts.orientation == ScrollOrientation::Horizontal {
                    self.scroll_by_fraction(1.0 / 5.0, ScrollUnit::Viewport);
                    return true;
                }
            }
            _ => {}
        }

        false
    }

    /// Draw a vertical scrollbar using the stored internal `slider` and arrows.
    ///
    /// Note: currently unused because `Widget::draw` rebuilds a render-only slider each frame to
    /// avoid mutating `self` from an `&self` method. Kept as a future hook if/when:
    /// - `Widget::draw` becomes `&mut self`, or
    /// - scrollbars move to a retained-mode UI tree, or
    /// - we add a separate pre-draw sync pass that updates the internal slider.
    #[allow(dead_code)]
    fn draw_vertical(&self, window: &mut dyn Window) -> Result<()> {
        // Layout: optional arrows at top/bottom, slider track in middle.
        if self.opts.show_arrows {
            // Top arrow
            {
                let mut view = WindowView {
                    window,
                    x_offset: 0,
                    y_offset: 0,
                    scroll_x: 0,
                    scroll_y: 0,
                    width: self.width.max(1),
                    height: 1,
                };
                self.start_arrow.draw(&mut view)?;
            }

            // Bottom arrow
            {
                let y = self.height.saturating_sub(1);
                let mut view = WindowView {
                    window,
                    x_offset: 0,
                    y_offset: y,
                    scroll_x: 0,
                    scroll_y: 0,
                    width: self.width.max(1),
                    height: 1,
                };
                self.end_arrow.draw(&mut view)?;
            }
        }

        // Slider
        {
            let top = if self.opts.show_arrows { 1 } else { 0 };
            let h = self
                .height
                .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                .max(1);

            let mut view = WindowView {
                window,
                x_offset: 0,
                y_offset: top,
                scroll_x: 0,
                scroll_y: 0,
                width: self.width.max(1),
                height: h,
            };
            self.slider.draw(&mut view)?;
        }

        Ok(())
    }

    /// Draw a horizontal scrollbar using the stored internal `slider` and arrows.
    ///
    /// Note: currently unused for the same reason as `draw_vertical` (see that comment).
    #[allow(dead_code)]
    fn draw_horizontal(&self, window: &mut dyn Window) -> Result<()> {
        if self.opts.show_arrows {
            // Left arrow
            {
                let mut view = WindowView {
                    window,
                    x_offset: 0,
                    y_offset: 0,
                    scroll_x: 0,
                    scroll_y: 0,
                    width: 1,
                    height: self.height.max(1),
                };
                self.start_arrow.draw(&mut view)?;
            }

            // Right arrow
            {
                let x = self.width.saturating_sub(1);
                let mut view = WindowView {
                    window,
                    x_offset: x,
                    y_offset: 0,
                    scroll_x: 0,
                    scroll_y: 0,
                    width: 1,
                    height: self.height.max(1),
                };
                self.end_arrow.draw(&mut view)?;
            }
        }

        // Slider
        {
            let left = if self.opts.show_arrows { 1 } else { 0 };
            let w = self
                .width
                .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                .max(1);

            let mut view = WindowView {
                window,
                x_offset: left,
                y_offset: 0,
                scroll_x: 0,
                scroll_y: 0,
                width: w,
                height: self.height.max(1),
            };
            self.slider.draw(&mut view)?;
        }

        Ok(())
    }
}

impl Widget for ScrollBar {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // If the app has called `sync_from_state_and_resize_parts()` this frame, render using the
        // internal slider/arrows so drag state is preserved.
        //
        // Otherwise, fall back to the previous "render-only rebuild" path for correctness.
        if self.synced {
            match self.opts.orientation {
                ScrollOrientation::Vertical => {
                    if self.opts.show_arrows {
                        {
                            let mut view = WindowView {
                                window,
                                x_offset: 0,
                                y_offset: 0,
                                scroll_x: 0,
                                scroll_y: 0,
                                width: self.width.max(1),
                                height: 1,
                            };
                            self.start_arrow.draw(&mut view)?;
                        }
                        {
                            let y = self.height.saturating_sub(1);
                            let mut view = WindowView {
                                window,
                                x_offset: 0,
                                y_offset: y,
                                scroll_x: 0,
                                scroll_y: 0,
                                width: self.width.max(1),
                                height: 1,
                            };
                            self.end_arrow.draw(&mut view)?;
                        }
                    }

                    let top = if self.opts.show_arrows { 1 } else { 0 };
                    let h = self
                        .height
                        .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                        .max(1);

                    let mut view = WindowView {
                        window,
                        x_offset: 0,
                        y_offset: top,
                        scroll_x: 0,
                        scroll_y: 0,
                        width: self.width.max(1),
                        height: h,
                    };
                    self.slider.draw(&mut view)?;
                }
                ScrollOrientation::Horizontal => {
                    if self.opts.show_arrows {
                        {
                            let mut view = WindowView {
                                window,
                                x_offset: 0,
                                y_offset: 0,
                                scroll_x: 0,
                                scroll_y: 0,
                                width: 1,
                                height: self.height.max(1),
                            };
                            self.start_arrow.draw(&mut view)?;
                        }
                        {
                            let x = self.width.saturating_sub(1);
                            let mut view = WindowView {
                                window,
                                x_offset: x,
                                y_offset: 0,
                                scroll_x: 0,
                                scroll_y: 0,
                                width: 1,
                                height: self.height.max(1),
                            };
                            self.end_arrow.draw(&mut view)?;
                        }
                    }

                    let left = if self.opts.show_arrows { 1 } else { 0 };
                    let w = self
                        .width
                        .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                        .max(1);

                    let mut view = WindowView {
                        window,
                        x_offset: left,
                        y_offset: 0,
                        scroll_x: 0,
                        scroll_y: 0,
                        width: w,
                        height: self.height.max(1),
                    };
                    self.slider.draw(&mut view)?;
                }
            }

            return Ok(());
        }

        // Fallback: ensure the slider reflects latest scroll state (in case other widgets modified it).
        // We can't mutate here (Widget::draw takes &self), so we do a lightweight render-only sync:
        // build a local slider configured from state and draw that.
        let s = *self.state.borrow();

        let max_offset = s.max_offset_for(self.opts.orientation);
        let viewport_size = match self.opts.orientation {
            ScrollOrientation::Vertical => s.viewport_size().height,
            ScrollOrientation::Horizontal => s.viewport_size().width,
        };
        let offset = s.offset_for(self.opts.orientation);

        let slider_orientation = match self.opts.orientation {
            ScrollOrientation::Vertical => SliderOrientation::Vertical,
            ScrollOrientation::Horizontal => SliderOrientation::Horizontal,
        };

        let slider_w;
        let slider_h;
        match self.opts.orientation {
            ScrollOrientation::Vertical => {
                slider_w = self.width.max(1);
                slider_h = self
                    .height
                    .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                    .max(1);
            }
            ScrollOrientation::Horizontal => {
                slider_w = self
                    .width
                    .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                    .max(1);
                slider_h = self.height.max(1);
            }
        }

        let render_slider = Slider::new(
            slider_w,
            slider_h,
            SliderOptions {
                orientation: slider_orientation,
                min: 0.0,
                max: max_offset as f32,
                value: offset as f32,
                viewport_size: viewport_size.max(1) as f32,
                track_color: self.opts.track_color,
                thumb_color: self.opts.thumb_color,
                min_thumb_virtual: 1,
            },
        );

        let start_arrow = match self.opts.orientation {
            ScrollOrientation::Vertical => ArrowButton {
                width: self.width.max(1),
                height: 1,
                dir: ArrowDirection::Up,
                color: self.opts.arrow_color,
            },
            ScrollOrientation::Horizontal => ArrowButton {
                width: 1,
                height: self.height.max(1),
                dir: ArrowDirection::Left,
                color: self.opts.arrow_color,
            },
        };

        let end_arrow = match self.opts.orientation {
            ScrollOrientation::Vertical => ArrowButton {
                width: self.width.max(1),
                height: 1,
                dir: ArrowDirection::Down,
                color: self.opts.arrow_color,
            },
            ScrollOrientation::Horizontal => ArrowButton {
                width: 1,
                height: self.height.max(1),
                dir: ArrowDirection::Right,
                color: self.opts.arrow_color,
            },
        };

        match self.opts.orientation {
            ScrollOrientation::Vertical => {
                if self.opts.show_arrows {
                    {
                        let mut view = WindowView {
                            window,
                            x_offset: 0,
                            y_offset: 0,
                            scroll_x: 0,
                            scroll_y: 0,
                            width: self.width.max(1),
                            height: 1,
                        };
                        start_arrow.draw(&mut view)?;
                    }
                    {
                        let y = self.height.saturating_sub(1);
                        let mut view = WindowView {
                            window,
                            x_offset: 0,
                            y_offset: y,
                            scroll_x: 0,
                            scroll_y: 0,
                            width: self.width.max(1),
                            height: 1,
                        };
                        end_arrow.draw(&mut view)?;
                    }
                }

                let top = if self.opts.show_arrows { 1 } else { 0 };
                let h = self
                    .height
                    .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                    .max(1);
                let mut view = WindowView {
                    window,
                    x_offset: 0,
                    y_offset: top,
                    scroll_x: 0,
                    scroll_y: 0,
                    width: self.width.max(1),
                    height: h,
                };
                render_slider.draw(&mut view)?;
            }
            ScrollOrientation::Horizontal => {
                if self.opts.show_arrows {
                    {
                        let mut view = WindowView {
                            window,
                            x_offset: 0,
                            y_offset: 0,
                            scroll_x: 0,
                            scroll_y: 0,
                            width: 1,
                            height: self.height.max(1),
                        };
                        start_arrow.draw(&mut view)?;
                    }
                    {
                        let x = self.width.saturating_sub(1);
                        let mut view = WindowView {
                            window,
                            x_offset: x,
                            y_offset: 0,
                            scroll_x: 0,
                            scroll_y: 0,
                            width: 1,
                            height: self.height.max(1),
                        };
                        end_arrow.draw(&mut view)?;
                    }
                }

                let left = if self.opts.show_arrows { 1 } else { 0 };
                let w = self
                    .width
                    .saturating_sub(if self.opts.show_arrows { 2 } else { 0 })
                    .max(1);
                let mut view = WindowView {
                    window,
                    x_offset: left,
                    y_offset: 0,
                    scroll_x: 0,
                    scroll_y: 0,
                    width: w,
                    height: self.height.max(1),
                };
                render_slider.draw(&mut view)?;
            }
        }

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (0, 0)
    }
}
