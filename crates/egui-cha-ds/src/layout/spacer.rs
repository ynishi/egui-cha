//! Spacer - Flexible space filler

use egui::Ui;

/// Flexible spacer for layouts
pub struct Spacer {
    size: SpacerSize,
}

enum SpacerSize {
    Fixed(f32),
    Flex,
}

impl Spacer {
    /// Create a fixed-size spacer
    pub fn fixed(size: f32) -> Self {
        Self {
            size: SpacerSize::Fixed(size),
        }
    }

    /// Create a flexible spacer that fills available space
    pub fn flex() -> Self {
        Self {
            size: SpacerSize::Flex,
        }
    }

    /// Show the spacer
    pub fn show(self, ui: &mut Ui) {
        match self.size {
            SpacerSize::Fixed(size) => {
                ui.add_space(size);
            }
            SpacerSize::Flex => {
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |_| {},
                );
            }
        }
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::flex()
    }
}

/// Convenience function for flexible spacer
pub fn spacer() -> impl FnOnce(&mut Ui) {
    |ui| Spacer::flex().show(ui)
}

/// Convenience function for fixed spacer
pub fn space(size: f32) -> impl FnOnce(&mut Ui) {
    move |ui| Spacer::fixed(size).show(ui)
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacer_fixed_creation() {
        let spacer = Spacer::fixed(20.0);
        // Verify it's a Fixed variant
        match spacer.size {
            SpacerSize::Fixed(size) => assert_eq!(size, 20.0),
            SpacerSize::Flex => panic!("Expected Fixed, got Flex"),
        }
    }

    #[test]
    fn spacer_flex_creation() {
        let spacer = Spacer::flex();
        // Verify it's a Flex variant
        match spacer.size {
            SpacerSize::Flex => {} // OK
            SpacerSize::Fixed(_) => panic!("Expected Flex, got Fixed"),
        }
    }

    #[test]
    fn spacer_default_is_flex() {
        let spacer = Spacer::default();
        match spacer.size {
            SpacerSize::Flex => {} // OK
            SpacerSize::Fixed(_) => panic!("Expected Flex, got Fixed"),
        }
    }

    #[test]
    fn spacer_convenience_functions_compile() {
        // Test that convenience functions return the expected type
        let _flex_fn = spacer();
        let _fixed_fn = space(10.0);
        // These are closures that can be used with .add()
    }

    #[test]
    fn spacer_fixed_zero() {
        let spacer = Spacer::fixed(0.0);
        match spacer.size {
            SpacerSize::Fixed(size) => assert_eq!(size, 0.0),
            SpacerSize::Flex => panic!("Expected Fixed, got Flex"),
        }
    }

    #[test]
    fn spacer_fixed_large_value() {
        let spacer = Spacer::fixed(1000.0);
        match spacer.size {
            SpacerSize::Fixed(size) => assert_eq!(size, 1000.0),
            SpacerSize::Flex => panic!("Expected Fixed, got Flex"),
        }
    }
}
