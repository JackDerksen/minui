use crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};
use minui::widgets::WidgetArea;
use minui::{Event, InteractionCache, MouseHandler, RouteTarget, UiScene};

#[test]
fn wheel_coordinates_route_without_movement_tracking() {
    for (kind, expected) in [
        (
            MouseEventKind::ScrollDown,
            Event::MouseScroll {
                x: 12,
                y: 7,
                delta: 1,
            },
        ),
        (
            MouseEventKind::ScrollUp,
            Event::MouseScroll {
                x: 12,
                y: 7,
                delta: -1,
            },
        ),
        (
            MouseEventKind::ScrollLeft,
            Event::MouseScrollHorizontal {
                x: 12,
                y: 7,
                delta: 1,
            },
        ),
        (
            MouseEventKind::ScrollRight,
            Event::MouseScrollHorizontal {
                x: 12,
                y: 7,
                delta: -1,
            },
        ),
    ] {
        let mut mouse = MouseHandler::new();
        mouse.set_movement_tracking(false);
        let event = mouse.process_mouse_event(MouseEvent {
            kind,
            column: 12,
            row: 7,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(event, expected);

        let mut scene = UiScene::new();
        scene.register_scrollable(1, WidgetArea::new(10, 5, 10, 5));
        assert_eq!(scene.route_wheel_event(&event), Some(RouteTarget::Id(1)));
        assert_eq!(scene.last_mouse_pos(), Some((12, 7)));

        // A stale hover position must not override the next wheel event's coordinates.
        scene.observe_event(&Event::MouseMove { x: 0, y: 0 });
        assert_eq!(scene.route_wheel_event(&event), Some(RouteTarget::Id(1)));

        let mut cache = InteractionCache::new();
        cache.observe_event(&event);
        assert_eq!(cache.last_mouse_pos(), Some((12, 7)));
    }
}

#[test]
fn wheel_routing_prefers_hit_then_scrollable_owner_then_focus() {
    let mut scene = UiScene::new();
    scene.register_scrollable(1, WidgetArea::new(0, 0, 20, 10));
    scene.register_focusable(2, WidgetArea::new(2, 2, 5, 5));
    scene.register_scrollable(3, WidgetArea::new(30, 0, 10, 10));
    scene.set_owner(2, 1);
    scene.focus(3);

    let event = Event::MouseScroll {
        x: 3,
        y: 3,
        delta: 1,
    };
    assert_eq!(scene.route_wheel_event(&event), Some(RouteTarget::Id(1)));
    assert_eq!(scene.cache().last_hovered(), Some(2));

    // A scrollable child takes precedence over its owner.
    scene.register_scrollable(2, WidgetArea::new(2, 2, 5, 5));
    assert_eq!(scene.route_wheel_event(&event), Some(RouteTarget::Id(2)));
    assert_eq!(
        scene.route_wheel_event_to_owner(&event),
        Some(RouteTarget::Owner(1))
    );

    scene.begin_frame();
    scene.register_focusable(1, WidgetArea::new(0, 0, 20, 10));
    scene.register_focusable(2, WidgetArea::new(2, 2, 5, 5));
    scene.register_scrollable(3, WidgetArea::new(30, 0, 10, 10));
    // A non-scrollable or missing owner cannot consume the wheel event.
    for owner in [1, 99] {
        scene.set_owner(2, owner);
        assert_eq!(scene.route_wheel_event(&event), Some(RouteTarget::Id(3)));
    }
    scene.clear_focus();
    assert_eq!(scene.route_wheel_event(&event), None);
    assert_eq!(
        scene.route_wheel_event(&Event::MouseMove { x: 3, y: 3 }),
        None
    );
}

#[test]
fn cross_axis_noise_is_dropped_and_switching_keeps_current_coordinates() {
    let mut mouse = MouseHandler::new();
    mouse.set_invert_scroll_vertical(true);
    mouse.set_invert_scroll_horizontal(true);

    for (kind, expected) in [
        (
            MouseEventKind::ScrollDown,
            Event::MouseScroll {
                x: 4,
                y: 6,
                delta: -1,
            },
        ),
        (MouseEventKind::ScrollLeft, Event::Unknown),
        // Returning to the active axis resets the pending switch.
        (
            MouseEventKind::ScrollUp,
            Event::MouseScroll {
                x: 4,
                y: 6,
                delta: 1,
            },
        ),
        (MouseEventKind::ScrollRight, Event::Unknown),
        (
            MouseEventKind::ScrollLeft,
            Event::MouseScrollHorizontal {
                x: 4,
                y: 6,
                delta: -1,
            },
        ),
        (MouseEventKind::ScrollDown, Event::Unknown),
        (
            MouseEventKind::ScrollRight,
            Event::MouseScrollHorizontal {
                x: 4,
                y: 6,
                delta: 1,
            },
        ),
        (MouseEventKind::ScrollDown, Event::Unknown),
        (
            MouseEventKind::ScrollUp,
            Event::MouseScroll {
                x: 4,
                y: 6,
                delta: 1,
            },
        ),
    ] {
        let noise = expected == Event::Unknown;
        let event = mouse.process_mouse_event(MouseEvent {
            kind,
            column: if noise { 1 } else { 4 },
            row: if noise { 2 } else { 6 },
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(event, expected);
    }
}

#[test]
fn axis_filtering_can_be_disabled_and_reenabled_without_stale_state() {
    let mut mouse = MouseHandler::new();
    assert!(mouse.is_scroll_axis_filtering_enabled());
    let scroll = |mouse: &mut MouseHandler, kind| {
        mouse.process_mouse_event(MouseEvent {
            kind,
            column: 12,
            row: 7,
            modifiers: KeyModifiers::NONE,
        })
    };
    scroll(&mut mouse, MouseEventKind::ScrollDown);
    assert_eq!(
        scroll(&mut mouse, MouseEventKind::ScrollLeft),
        Event::Unknown
    );

    mouse.set_scroll_axis_filtering(false);
    assert!(!mouse.is_scroll_axis_filtering_enabled());
    mouse.set_invert_scroll_vertical(true);
    mouse.set_invert_scroll_horizontal(true);
    let vertical = Event::MouseScroll {
        x: 12,
        y: 7,
        delta: -1,
    };
    let horizontal = Event::MouseScrollHorizontal {
        x: 12,
        y: 7,
        delta: -1,
    };
    for _ in 0..2 {
        assert_eq!(scroll(&mut mouse, MouseEventKind::ScrollLeft), horizontal);
        assert_eq!(scroll(&mut mouse, MouseEventKind::ScrollDown), vertical);
    }

    mouse.set_scroll_axis_filtering(true);
    assert!(mouse.is_scroll_axis_filtering_enabled());
    assert_eq!(scroll(&mut mouse, MouseEventKind::ScrollLeft), horizontal);
    assert_eq!(
        scroll(&mut mouse, MouseEventKind::ScrollDown),
        Event::Unknown
    );
    // Setting the same value again must not reset a pending axis switch.
    mouse.set_scroll_axis_filtering(true);
    assert_eq!(scroll(&mut mouse, MouseEventKind::ScrollDown), vertical);
}
