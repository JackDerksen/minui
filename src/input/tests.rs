//! Tests for the input module
//!
//! These tests verify the basic functionality of keyboard and mouse input handlers,
//! including keybind management and event processing.

#[cfg(test)]
mod tests {

    use crate::input::{KeybindAction, KeyboardHandler, MouseHandler};
    use std::time::Duration;

    #[test]
    fn test_keyboard_handler_creation() {
        let keyboard = KeyboardHandler::new();
        assert_eq!(keyboard.poll_rate(), Duration::from_millis(1));
        assert_eq!(keyboard.keybinds().len(), 0);
    }

    #[test]
    fn test_keyboard_handler_with_common_keybinds() {
        let keyboard = KeyboardHandler::with_common_keybinds();
        assert!(keyboard.keybinds().len() > 0);

        // Should have standard keybinds
        let keybinds = keyboard.keybinds();
        assert!(
            keybinds
                .values()
                .any(|action| matches!(action, KeybindAction::Quit))
        );
        assert!(
            keybinds
                .values()
                .any(|action| matches!(action, KeybindAction::Save))
        );
        assert!(
            keybinds
                .values()
                .any(|action| matches!(action, KeybindAction::Cut))
        );
    }

    #[test]
    fn test_keybind_management() {
        let mut keyboard = KeyboardHandler::new();

        // Test adding keybinds
        match keyboard.add_keybind("ctrl-c", KeybindAction::Copy) {
            Ok(_) => {}
            Err(e) => panic!("Failed to add keybind: {}", e),
        }
        match keyboard.add_keybind("f5", KeybindAction::Custom("refresh".to_string())) {
            Ok(_) => {}
            Err(e) => panic!("Failed to add F5 keybind: {}", e),
        }
        assert_eq!(keyboard.keybinds().len(), 2);

        // Test invalid keybind
        assert!(
            keyboard
                .add_keybind("invalid-key-combo", KeybindAction::Save)
                .is_err()
        );

        // Test removing keybinds
        assert!(keyboard.remove_keybind("ctrl-c").is_ok());
        assert_eq!(keyboard.keybinds().len(), 1);

        // Test removing non-existent keybind
        let removed = keyboard.remove_keybind("ctrl-z").unwrap();
        assert!(!removed);

        // Test clearing keybinds
        keyboard.clear_keybinds();
        assert_eq!(keyboard.keybinds().len(), 0);
    }

    #[test]
    fn test_keyboard_poll_rate() {
        let mut keyboard = KeyboardHandler::new();

        keyboard.set_poll_rate(16);
        assert_eq!(keyboard.poll_rate(), Duration::from_millis(16));

        keyboard.set_poll_rate(1);
        assert_eq!(keyboard.poll_rate(), Duration::from_millis(1));
    }

    #[test]
    fn test_mouse_handler_creation() {
        let mouse = MouseHandler::new();
        assert_eq!(mouse.poll_rate(), Duration::from_millis(1));
        assert!(mouse.is_movement_tracking_enabled());
        assert!(!mouse.is_drag_detection_enabled());
        assert!(!mouse.is_dragging());
        assert_eq!(mouse.drag_start_position(), None);
    }

    #[test]
    fn test_mouse_configuration() {
        let mut mouse = MouseHandler::new();

        // Test poll rate
        mouse.set_poll_rate(16);
        assert_eq!(mouse.poll_rate(), Duration::from_millis(16));

        // Test movement tracking
        mouse.set_movement_tracking(false);
        assert!(!mouse.is_movement_tracking_enabled());

        mouse.set_movement_tracking(true);
        assert!(mouse.is_movement_tracking_enabled());

        // Test drag detection
        mouse.enable_drag_detection(true);
        assert!(mouse.is_drag_detection_enabled());

        mouse.enable_drag_detection(false);
        assert!(!mouse.is_drag_detection_enabled());
        assert!(!mouse.is_dragging()); // Should reset drag state
    }

    #[test]
    fn test_keybind_action_variants() {
        let actions = vec![
            KeybindAction::Quit,
            KeybindAction::Save,
            KeybindAction::Copy,
            KeybindAction::Paste,
            KeybindAction::Cut,
            KeybindAction::Undo,
            KeybindAction::Redo,
            KeybindAction::SelectAll,
            KeybindAction::Find,
            KeybindAction::Replace,
            KeybindAction::New,
            KeybindAction::Open,
            KeybindAction::Custom("test".to_string()),
        ];

        // Test that all variants can be created and compared
        for action in actions {
            match action {
                KeybindAction::Custom(ref s) => assert_eq!(s, "test"),
                _ => {} // All other variants are fine
            }
        }
    }

    #[test]
    fn test_combined_input_handler() {
        use crate::input::CombinedInputHandler;

        let mut combined = CombinedInputHandler::new();

        // Should be able to configure keyboard
        match combined
            .keyboard_mut()
            .add_keybind("ctrl-c", KeybindAction::Quit)
        {
            Ok(_) => {}
            Err(e) => panic!("Failed to add combined keybind: {}", e),
        }

        // Should be able to configure mouse
        combined.mouse_mut().enable_drag_detection(true);
        assert!(combined.mouse_mut().is_drag_detection_enabled());

        // Test with common keybinds
        let mut combined_common = CombinedInputHandler::with_common_keybinds();
        assert!(combined_common.keyboard_mut().keybinds().len() > 0);
    }

    #[test]
    fn test_keybind_action_debug() {
        let action = KeybindAction::Custom("test_action".to_string());
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("Custom"));
        assert!(debug_str.contains("test_action"));
    }

    #[test]
    fn test_keybind_action_clone() {
        let original = KeybindAction::Custom("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_default_implementations() {
        let keyboard = KeyboardHandler::default();
        assert_eq!(keyboard.poll_rate(), Duration::from_millis(1));

        let mouse = MouseHandler::default();
        assert_eq!(mouse.poll_rate(), Duration::from_millis(1));

        let mut combined = crate::input::CombinedInputHandler::default();
        // Just test that it can be created
        assert_eq!(combined.keyboard_mut().keybinds().len(), 0);
    }
}
