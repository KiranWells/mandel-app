use std::hint::unreachable_unchecked;

/// converts hsl to rgb, modified from
/// https://web.archive.org/web/20081227003853/http://mjijackson.com/2008/02/rgb-to-hsl-and-rgb-to-hsv-color-model-conversion-algorithms-in-javascript
pub fn hsl2rgb(h: f64, s: f64, v: f64) -> [u8; 3] {
    let r;
    let g;
    let b;

    let i = (h * 6.).floor();
    let f = h * 6. - i;
    let p = v * (1. - s);
    let q = v * (1. - f * s);
    let t = v * (1. - (1. - f) * s);

    match (i % 6.0) as u8 {
        0 => {
            r = v;
            g = t;
            b = p;
        }
        1 => {
            r = q;
            g = v;
            b = p;
        }
        2 => {
            r = p;
            g = v;
            b = t;
        }
        3 => {
            r = p;
            g = q;
            b = v;
        }
        4 => {
            r = t;
            g = p;
            b = v;
        }
        5 => {
            r = v;
            g = p;
            b = q;
        }
        _ => unsafe { unreachable_unchecked() },
    }

    [(r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8]
}
