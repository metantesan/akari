use akari_core::Icon;

pub(super) fn flatten_icon(icon: Icon) -> Vec<[f32; 4]> {
    let mut segments = Vec::new();
    for data in icon.paths {
        flatten_svg_path(data, &mut segments);
    }
    for &(cx, cy, radius) in icon.circles {
        let mut previous = [cx + radius, cy];
        for step in 1..=48 {
            let angle = std::f32::consts::TAU * step as f32 / 48.;
            let next = [cx + radius * angle.cos(), cy + radius * angle.sin()];
            segments.push([previous[0], previous[1], next[0], next[1]]);
            previous = next;
        }
    }
    segments
}

#[derive(Clone, Copy)]
enum PathToken {
    Command(u8),
    Number(f32),
}

pub(super) fn flatten_svg_path(data: &str, out: &mut Vec<[f32; 4]>) {
    let bytes = data.as_bytes();
    let mut tokens = Vec::new();
    let mut pos = 0;
    let mut command = 0_u8;
    let mut arc_argument = 0_usize;
    while pos < bytes.len() {
        let b = bytes[pos];
        if b.is_ascii_whitespace() || b == b',' {
            pos += 1;
        } else if b.is_ascii_alphabetic() {
            tokens.push(PathToken::Command(b));
            command = b;
            arc_argument = 0;
            pos += 1;
        } else {
            if command.eq_ignore_ascii_case(&b'A') && matches!(arc_argument % 7, 3 | 4) {
                if !matches!(b, b'0' | b'1') {
                    return;
                }
                tokens.push(PathToken::Number((b - b'0') as f32));
                arc_argument += 1;
                pos += 1;
                continue;
            }
            let start = pos;
            if matches!(bytes[pos], b'+' | b'-') {
                pos += 1;
            }
            while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                pos += 1;
            }
            if pos < bytes.len() && bytes[pos] == b'.' {
                pos += 1;
                while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            if pos < bytes.len() && matches!(bytes[pos], b'e' | b'E') {
                pos += 1;
                if pos < bytes.len() && matches!(bytes[pos], b'+' | b'-') {
                    pos += 1;
                }
                while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            if start == pos {
                return;
            }
            let Ok(number) = data[start..pos].parse::<f32>() else {
                return;
            };
            tokens.push(PathToken::Number(number));
            if command.eq_ignore_ascii_case(&b'A') {
                arc_argument += 1;
            }
        }
    }

    let (mut x, mut y, mut sx, mut sy) = (0_f32, 0_f32, 0_f32, 0_f32);
    let (mut last_cubic, mut last_quadratic) = (None, None);
    let mut command = 0_u8;
    let mut i = 0;
    while i < tokens.len() {
        if let PathToken::Command(c) = tokens[i] {
            command = c;
            i += 1;
        } else if command == 0 {
            return;
        }
        let relative = command.is_ascii_lowercase();
        match command.to_ascii_uppercase() {
            b'M' => {
                let mut first = true;
                while matches!(tokens.get(i), Some(PathToken::Number(_))) {
                    let Some((mut nx, mut ny)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    if relative {
                        nx += x;
                        ny += y;
                    }
                    if first {
                        x = nx;
                        y = ny;
                        sx = nx;
                        sy = ny;
                        first = false;
                        command = if relative { b'l' } else { b'L' };
                    } else {
                        out.push([x, y, nx, ny]);
                        x = nx;
                        y = ny;
                    }
                }
                last_cubic = None;
                last_quadratic = None;
            }
            b'L' => {
                let mut any = false;
                while matches!(tokens.get(i), Some(PathToken::Number(_))) {
                    let Some((mut nx, mut ny)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    if relative {
                        nx += x;
                        ny += y;
                    }
                    out.push([x, y, nx, ny]);
                    x = nx;
                    y = ny;
                    any = true;
                }
                if !any {
                    return;
                }
                last_cubic = None;
                last_quadratic = None;
            }
            b'H' | b'V' => {
                let mut any = false;
                while matches!(tokens.get(i), Some(PathToken::Number(_))) {
                    let Some(n) = path_number(&tokens, &mut i) else {
                        return;
                    };
                    let (nx, ny) = if command.eq_ignore_ascii_case(&b'H') {
                        (if relative { x + n } else { n }, y)
                    } else {
                        (x, if relative { y + n } else { n })
                    };
                    out.push([x, y, nx, ny]);
                    x = nx;
                    y = ny;
                    any = true;
                }
                if !any {
                    return;
                }
                last_cubic = None;
                last_quadratic = None;
            }
            b'C' => {
                let mut any = false;
                while matches!(tokens.get(i), Some(PathToken::Number(_))) {
                    let Some((mut x1, mut y1)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    let Some((mut x2, mut y2)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    let Some((mut nx, mut ny)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    if relative {
                        x1 += x;
                        y1 += y;
                        x2 += x;
                        y2 += y;
                        nx += x;
                        ny += y;
                    }
                    flatten_cubic(out, [x, y], [x1, y1], [x2, y2], [nx, ny]);
                    x = nx;
                    y = ny;
                    last_cubic = Some([x2, y2]);
                    last_quadratic = None;
                    any = true;
                }
                if !any {
                    return;
                }
            }
            b'S' => {
                let mut any = false;
                while matches!(tokens.get(i), Some(PathToken::Number(_))) {
                    let Some((mut x2, mut y2)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    let Some((mut nx, mut ny)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    if relative {
                        x2 += x;
                        y2 += y;
                        nx += x;
                        ny += y;
                    }
                    let [x1, y1] = last_cubic.map_or([x, y], |[cx, cy]| [2. * x - cx, 2. * y - cy]);
                    flatten_cubic(out, [x, y], [x1, y1], [x2, y2], [nx, ny]);
                    x = nx;
                    y = ny;
                    last_cubic = Some([x2, y2]);
                    last_quadratic = None;
                    any = true;
                }
                if !any {
                    return;
                }
            }
            b'Q' => {
                let mut any = false;
                while matches!(tokens.get(i), Some(PathToken::Number(_))) {
                    let Some((mut qx, mut qy)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    let Some((mut nx, mut ny)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    if relative {
                        qx += x;
                        qy += y;
                        nx += x;
                        ny += y;
                    }
                    flatten_quadratic(out, [x, y], [qx, qy], [nx, ny]);
                    x = nx;
                    y = ny;
                    last_quadratic = Some([qx, qy]);
                    last_cubic = None;
                    any = true;
                }
                if !any {
                    return;
                }
            }
            b'T' => {
                let mut any = false;
                while matches!(tokens.get(i), Some(PathToken::Number(_))) {
                    let Some((mut nx, mut ny)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    if relative {
                        nx += x;
                        ny += y;
                    }
                    let [qx, qy] =
                        last_quadratic.map_or([x, y], |[cx, cy]| [2. * x - cx, 2. * y - cy]);
                    flatten_quadratic(out, [x, y], [qx, qy], [nx, ny]);
                    x = nx;
                    y = ny;
                    last_quadratic = Some([qx, qy]);
                    last_cubic = None;
                    any = true;
                }
                if !any {
                    return;
                }
            }
            b'A' => {
                let mut any = false;
                while matches!(tokens.get(i), Some(PathToken::Number(_))) {
                    let Some(rx) = path_number(&tokens, &mut i) else {
                        return;
                    };
                    let Some(ry) = path_number(&tokens, &mut i) else {
                        return;
                    };
                    let Some(rotation) = path_number(&tokens, &mut i) else {
                        return;
                    };
                    let Some(large) = path_number(&tokens, &mut i) else {
                        return;
                    };
                    let Some(sweep) = path_number(&tokens, &mut i) else {
                        return;
                    };
                    let Some((mut nx, mut ny)) = path_pair(&tokens, &mut i) else {
                        return;
                    };
                    if relative {
                        nx += x;
                        ny += y;
                    }
                    append_svg_arc(
                        out,
                        [x, y],
                        [rx, ry],
                        rotation,
                        large != 0.,
                        sweep != 0.,
                        [nx, ny],
                    );
                    x = nx;
                    y = ny;
                    last_cubic = None;
                    last_quadratic = None;
                    any = true;
                }
                if !any {
                    return;
                }
            }
            b'Z' => {
                out.push([x, y, sx, sy]);
                x = sx;
                y = sy;
                last_cubic = None;
                last_quadratic = None;
                command = 0;
            }
            _ => return,
        }
    }
}

fn flatten_cubic(
    out: &mut Vec<[f32; 4]>,
    start: [f32; 2],
    c1: [f32; 2],
    c2: [f32; 2],
    end: [f32; 2],
) {
    let mut previous = start;
    for step in 1..=12 {
        let t = step as f32 / 12.;
        let u = 1. - t;
        let point = [
            u.powi(3) * start[0]
                + 3. * u.powi(2) * t * c1[0]
                + 3. * u * t.powi(2) * c2[0]
                + t.powi(3) * end[0],
            u.powi(3) * start[1]
                + 3. * u.powi(2) * t * c1[1]
                + 3. * u * t.powi(2) * c2[1]
                + t.powi(3) * end[1],
        ];
        out.push([previous[0], previous[1], point[0], point[1]]);
        previous = point;
    }
}

fn flatten_quadratic(out: &mut Vec<[f32; 4]>, start: [f32; 2], control: [f32; 2], end: [f32; 2]) {
    let mut previous = start;
    for step in 1..=10 {
        let t = step as f32 / 10.;
        let u = 1. - t;
        let point = [
            u * u * start[0] + 2. * u * t * control[0] + t * t * end[0],
            u * u * start[1] + 2. * u * t * control[1] + t * t * end[1],
        ];
        out.push([previous[0], previous[1], point[0], point[1]]);
        previous = point;
    }
}

fn path_number(tokens: &[PathToken], i: &mut usize) -> Option<f32> {
    match tokens.get(*i).copied()? {
        PathToken::Number(n) => {
            *i += 1;
            Some(n)
        }
        PathToken::Command(_) => None,
    }
}

fn path_pair(tokens: &[PathToken], i: &mut usize) -> Option<(f32, f32)> {
    Some((path_number(tokens, i)?, path_number(tokens, i)?))
}

fn append_svg_arc(
    out: &mut Vec<[f32; 4]>,
    from: [f32; 2],
    radii: [f32; 2],
    rotation: f32,
    large_arc: bool,
    sweep: bool,
    to: [f32; 2],
) {
    use std::f64::consts::{PI, TAU};
    let (mut rx, mut ry) = (radii[0].abs() as f64, radii[1].abs() as f64);
    let (x1, y1, x2, y2) = (from[0] as f64, from[1] as f64, to[0] as f64, to[1] as f64);
    if rx == 0. || ry == 0. || (x1 == x2 && y1 == y2) {
        out.push([from[0], from[1], to[0], to[1]]);
        return;
    }
    let phi = (rotation as f64).to_radians();
    let (cos_phi, sin_phi) = (phi.cos(), phi.sin());
    let dx = (x1 - x2) * 0.5;
    let dy = (y1 - y2) * 0.5;
    let xp = cos_phi * dx + sin_phi * dy;
    let yp = -sin_phi * dx + cos_phi * dy;
    let lambda = xp * xp / (rx * rx) + yp * yp / (ry * ry);
    if lambda > 1. {
        let factor = lambda.sqrt();
        rx *= factor;
        ry *= factor;
    }
    let numerator = (rx * rx * ry * ry - rx * rx * yp * yp - ry * ry * xp * xp).max(0.);
    let denominator = rx * rx * yp * yp + ry * ry * xp * xp;
    let sign = if large_arc == sweep { -1. } else { 1. };
    let factor = if denominator == 0. {
        0.
    } else {
        sign * (numerator / denominator).sqrt()
    };
    let cxp = factor * rx * yp / ry;
    let cyp = factor * -ry * xp / rx;
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) * 0.5;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) * 0.5;
    let ux = (xp - cxp) / rx;
    let uy = (yp - cyp) / ry;
    let vx = (-xp - cxp) / rx;
    let vy = (-yp - cyp) / ry;
    let start = uy.atan2(ux);
    let mut delta = (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    if !sweep && delta > 0. {
        delta -= TAU;
    }
    if sweep && delta < 0. {
        delta += TAU;
    }
    let count = (delta.abs() / (PI / 24.)).ceil().max(1.) as usize;
    let point = |t: f64| {
        [
            (cx + cos_phi * rx * t.cos() - sin_phi * ry * t.sin()) as f32,
            (cy + sin_phi * rx * t.cos() + cos_phi * ry * t.sin()) as f32,
        ]
    };
    let mut previous = from;
    for n in 1..=count {
        let t = start + delta * n as f64 / count as f64;
        let next = if n == count { to } else { point(t) };
        out.push([previous[0], previous[1], next[0], next[1]]);
        previous = next;
    }
}
