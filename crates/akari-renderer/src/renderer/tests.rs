use super::*;
use crate::instance_descriptor;
use unicode_segmentation::UnicodeSegmentation;
#[test]
fn every_generated_lucide_icon_flattens_to_drawable_segments() {
    let icons = akari_icons::lucide::ALL_ICONS;
    assert_eq!(icons.len(), akari_icons::lucide::ICON_COUNT);
    for icon in icons {
        let mut segments = Vec::new();
        for path in icon.paths {
            let previous = segments.len();
            flatten_svg_path(path, &mut segments);
            assert!(
                segments.len() > previous,
                "{} contains a path the renderer could not flatten: {path}",
                icon.name
            );
        }
        for &(x, y, radius) in icon.circles {
            let mut previous = [x + radius, y];
            for step in 1..=48 {
                let angle = std::f32::consts::TAU * step as f32 / 48.;
                let next = [x + radius * angle.cos(), y + radius * angle.sin()];
                segments.push([previous[0], previous[1], next[0], next[1]]);
                previous = next;
            }
        }
        assert!(
            !segments.is_empty(),
            "{} has no drawable geometry",
            icon.name
        );
    }
}

#[test]
fn lucide_path_flattener_handles_cubic_quadratic_smooth_and_arc_segments() {
    let mut segments = Vec::new();
    flatten_svg_path(
        "M1 1 C1 8 8 8 8 1 S15 -6 15 1 Q18 4 15 7 T15 13 A3 3 0 0 1 9 13 Z",
        &mut segments,
    );
    assert_eq!(
        segments.first().map(|segment| [segment[0], segment[1]]),
        Some([1., 1.])
    );
    assert!(segments[0][2] > 1. && segments[0][2] < 2.);
    assert!(segments.len() > 40);
    assert_eq!(
        segments.last().map(|segment| [segment[2], segment[3]]),
        Some([1., 1.])
    );

    let mut compact_arc = Vec::new();
    flatten_svg_path("M10.5 2a34.5 34.5 0 000 20", &mut compact_arc);
    assert!(!compact_arc.is_empty(), "compact SVG arc flags must parse");
    assert_eq!(
        compact_arc.last().map(|segment| [segment[2], segment[3]]),
        Some([10.5, 22.])
    );
}

#[test]
#[ignore = "requires a GPU or software Vulkan adapter; run explicitly"]
fn gpu_composition_glass_resize_and_dpi() {
    let instance = wgpu::Instance::new(instance_descriptor());
    let mut r = pollster::block_on(Renderer::new(&instance, None)).expect("GPU adapter required");
    let size = [640, 480];
    let output = r.offscreen_texture(size);
    let full = Rect::new(0., 0., 640., 480.);
    let clip = Rect::new(40., 40., 560., 400.);
    let theme = Theme::light();
    let scene = vec![
        Draw::Quad {
            rect: Rect::new(50., 65., 280., 200.),
            clip,
            color: [0.08, 0.3, 0.8, 1.],
            radius: 0.,
            border: 0.,
        },
        Draw::Quad {
            rect: Rect::new(260., 100., 280., 200.),
            clip,
            color: [0.8, 0.15, 0.4, 0.6],
            radius: 24.,
            border: 0.,
        },
    ];
    let view = output.create_view(&Default::default());
    r.render(&view, size, 1., theme, &scene)
        .expect("value must be present");
    let base = r.read_rgba(&output, size);
    let mut glass_scene = scene.clone();
    glass_scene.push(Draw::Glass {
        rect: Rect::new(120., 120., 400., 210.),
        clip: full,
        style: akari_core::Glass {
            blur: 18.,
            refraction: 9.,
            ..Default::default()
        },
    });
    r.render(&view, size, 1., theme, &glass_scene)
        .expect("value must be present");
    let glass = r.read_rgba(&output, size);
    let at = |x: usize, y: usize| (y * 640 + x) * 4;
    assert_eq!(
        &base[at(5, 5)..at(5, 5) + 4],
        &glass[at(5, 5)..at(5, 5) + 4],
        "glass must not affect outside pixels"
    );
    assert_ne!(
        &base[at(300, 200)..at(300, 200) + 4],
        &glass[at(300, 200)..at(300, 200) + 4],
        "glass must change its backdrop"
    );
    let mut no_optics = glass_scene.clone();
    if let Draw::Glass { style, .. } = no_optics.last_mut().expect("value must be present") {
        style.blur = 0.;
        style.refraction = 0.;
    }
    r.render(&view, size, 1., theme, &no_optics)
        .expect("value must be present");
    let flat = r.read_rgba(&output, size);
    assert_ne!(
        flat, glass,
        "custom optical shader must affect the captured layers"
    );
    glass_scene.push(Draw::Text {
        rect: Rect::new(150., 180., 350., 90.),
        clip: full,
        text: "Akari liquid glass\nسلام، دنیا — Hello".into(),
        size: 24.,
        color: theme.text,
        align: TextAlign::Left,
    });
    glass_scene.push(Draw::Icon {
        rect: Rect::new(590., 80., 32., 32.),
        clip: full,
        icon: Icon {
            name: "test/plus",
            paths: &["M12 5v14", "M5 12h14"],
            circles: &[],
        },
        color: theme.accent,
    });
    let long_input = "A long title stays on one line without covering the next button";
    glass_scene.push(Draw::Quad {
        rect: Rect::new(420., 420., 190., 34.),
        clip: full,
        color: theme.surface,
        radius: 8.,
        border: 0.,
    });
    glass_scene.push(Draw::TextField {
        rect: Rect::new(428., 425., 174., 24.),
        clip: Rect::new(422., 422., 186., 30.),
        text: long_input.into(),
        size: 16.,
        color: theme.text,
        align: TextAlign::Left,
        state: akari_core::WidgetState {
            cursor: long_input.len(),
            anchor: long_input.len(),
            ..Default::default()
        },
        focused: true,
    });
    r.render(&view, size, 1., theme, &glass_scene)
        .expect("value must be present");
    let image = r.read_rgba(&output, size);
    assert_ne!(image, glass, "text must rasterize");
    assert_ne!(
        &image[at(606, 96)..at(606, 96) + 4],
        &image[at(5, 5)..at(5, 5) + 4],
        "Lucide path segments must draw through the Akari WGSL pipeline"
    );
    let long_text = "A task title that is longer than the available input width";
    assert_eq!(
        r.text_hit(long_text, 16., 90., 89., 10., (long_text.len(), false))
            .0,
        long_text.len(),
        "the insertion cursor should remain reachable after horizontal scrolling"
    );
    let width = 300.;
    for mixed in [
        "برنامه فردا — project update 123",
        "Project update 123 — فارسی e\u{301}",
    ] {
        let left_hit = r.text_hit(mixed, 16., width, 3., 10., (mixed.len(), false));
        let right_hit = r.text_hit(mixed, 16., width, width - 3., 10., (mixed.len(), false));
        assert_ne!(
            left_hit.0, right_hit.0,
            "mixed RTL/LTR runs must remain separately hit-testable"
        );
        let mut boundaries: Vec<_> = mixed
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .collect();
        boundaries.push(mixed.len());
        let mut candidates: Vec<_> = boundaries
            .iter()
            .flat_map(|index| [false, true].map(|after| (*index, after)))
            .filter_map(|cursor| {
                r.text_cursor_x(mixed, 16., width, cursor.0, cursor.1)
                    .map(|x| (cursor, x))
            })
            .collect();
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
        let (mut cursor, mut previous_x) = candidates[0];
        let mut visited = std::collections::HashSet::new();
        visited.insert(cursor);
        for _ in 0..boundaries.len() * 2 {
            let Some(next) = r.text_move(mixed, 16., width, cursor.0, cursor.1, true) else {
                break;
            };
            if next == cursor {
                break;
            }
            let next_x = r
                .text_cursor_x(mixed, 16., width, next.0, next.1)
                .expect("moved bidi cursor position");
            assert!(next_x >= previous_x, "ArrowRight follows visual x order");
            cursor = next;
            previous_x = next_x;
            visited.insert(cursor);
        }
        assert!(visited.len() >= boundaries.len());
        assert!(previous_x > candidates[0].1);
        let rightmost = cursor;
        let mut reverse_x = previous_x;
        for _ in 0..boundaries.len() * 2 {
            let Some(next) = r.text_move(mixed, 16., width, cursor.0, cursor.1, false) else {
                break;
            };
            if next == cursor {
                break;
            }
            let next_x = r
                .text_cursor_x(mixed, 16., width, next.0, next.1)
                .expect("moved bidi cursor position");
            assert!(next_x <= reverse_x, "ArrowLeft follows visual x order");
            cursor = next;
            reverse_x = next_x;
        }
        assert_eq!(cursor, candidates[0].0);
        assert_ne!(rightmost, cursor);
    }
    assert!(!r.glyphs.is_empty());
    let mut ppm = format!("P6\n{} {}\n255\n", size[0], size[1]).into_bytes();
    ppm.extend(image.chunks(4).flat_map(|p| p[..3].iter().copied()));
    std::fs::write("/tmp/akari-glass.ppm", ppm).expect("value must be present");
    for scale in [2., 1., 1.5] {
        let size = [320, 240];
        let output = r.offscreen_texture(size);
        r.render(
            &output.create_view(&Default::default()),
            size,
            scale,
            theme,
            &glass_scene,
        )
        .expect("value must be present");
        r.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("value must be present");
    }
}
