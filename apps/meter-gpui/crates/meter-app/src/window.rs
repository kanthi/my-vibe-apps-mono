//! Picks the main window's size from the screen it opens on, so it fits small displays and does not
//! look lost on large ones. Sizes are in points. The display bounds GPUI reports include the menu
//! bar and the Dock, so room is left for both.

pub const MIN_WIDTH: f32 = 780.0;
pub const MIN_HEIGHT: f32 = 480.0;

/// Space the menu bar and the Dock take together, plus a margin.
const CHROME: f32 = 170.0;

/// (width, height) for a display of `display_w` x `display_h` points.
pub fn initial_size(display_w: f32, display_h: f32) -> (f32, f32) {
    // Roughly four fifths of the screen, within sensible limits.
    let mut w = (display_w * 0.78).clamp(MIN_WIDTH, 1200.0);
    let mut h = (display_h * 0.78).clamp(MIN_HEIGHT, 820.0);
    // Never taller than what is visible between the menu bar and the Dock.
    h = h.min(display_h - CHROME).max(420.0);
    // Never wider than the display itself, even when the display is smaller than the minimum.
    w = w.min(display_w * 0.95);
    h = h.min(display_h * 0.95);
    (w.round(), h.round())
}

/// The smallest the user may shrink the window to. Never larger than the window starts at.
pub fn min_size(initial: (f32, f32)) -> (f32, f32) {
    (MIN_WIDTH.min(initial.0), MIN_HEIGHT.min(initial.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_laptop_display_fits_between_menu_bar_and_dock() {
        let (w, h) = initial_size(1280.0, 800.0);
        assert_eq!((w, h), (998.0, 624.0));
        assert!(h + CHROME <= 800.0 + 24.0, "leaves room for the menu bar and the Dock");
    }

    #[test]
    fn typical_and_large_displays_scale_then_cap() {
        assert_eq!(initial_size(1440.0, 900.0), (1123.0, 702.0));
        assert_eq!(initial_size(1920.0, 1080.0), (1200.0, 820.0)); // capped
        assert_eq!(initial_size(3008.0, 1692.0), (1200.0, 820.0)); // capped on a large display
    }

    #[test]
    fn tiny_displays_never_overflow() {
        let (w, h) = initial_size(1024.0, 640.0);
        assert!(w <= 1024.0 * 0.95 && h <= 640.0 * 0.95, "{w}x{h}");
        let (w, h) = initial_size(600.0, 400.0);
        assert!(w <= 600.0 && h <= 400.0, "{w}x{h}");
    }

    #[test]
    fn min_size_never_exceeds_the_starting_size() {
        assert_eq!(min_size((998.0, 624.0)), (780.0, 480.0));
        assert_eq!(min_size((570.0, 380.0)), (570.0, 380.0));
    }
}
