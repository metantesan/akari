//! Tree-shakeable Lucide icon data for Akari.
//!
//! Every checked-in Lucide SVG is converted into a per-icon module by `build.rs`.
//! Reference only the constants an app uses so unused icon data can be omitted
//! from the final executable.

pub use akari_core::Icon;

pub mod lucide {
    include!(concat!(env!("OUT_DIR"), "/lucide.rs"));
}

#[cfg(test)]
mod tests {
    use super::lucide;

    #[test]
    fn build_script_generates_the_vendored_lucide_collection() {
        assert_eq!(lucide::ICON_COUNT, 2_015);
        assert_eq!(lucide::activity.name, "lucide/activity");
        assert_eq!(lucide::circle_check.name, "lucide/circle-check");
        assert_eq!(lucide::check_circle.name, "lucide/circle-check");
        assert_eq!(lucide::type_.name, "lucide/type");
        assert!(!lucide::alarm_clock.paths.is_empty());
    }
}
