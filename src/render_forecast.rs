use std::ops::Range;

use quantogram::Quantogram;

use crate::svg::{Coord, Path, Segment, Svg};

pub fn render_dist(start: &Quantogram, end: &Quantogram, days: &Range<u32>) -> Svg {
    const H: f64 = 100.0;

    let mut coords: Vec<(f64, f64)> = vec![];

    // For every day, sample the probability that we are working on that day
    // The probability is P(working|day) = P(start <= day) * (1 - P(end <= day))

    for day in days.start..=days.end {
        // Take the lower quantile for the lhs, upper quantile for the rhs
        let started = start.quantile_at(day as f64).unwrap();
        let stopped = end.quantile_at(day as f64).unwrap();

        let right = started.1 * (1.0 - stopped.1);

        coords.push((day as f64, right));
    }

    let max_y = coords
        .iter()
        .map(|(_, y)| y)
        .fold(0.0, |a, y| if *y > a { *y } else { a });

    let mut path: Path = Vec::with_capacity(13);
    path.push(Segment::Move(0 as Coord, (H - 0.0 * H) as Coord));

    for (day, y) in coords {
        path.push(Segment::Line(day as Coord, (H - (y / max_y) * H) as Coord));
    }

    path.push(Segment::Line(days.end as Coord, H as Coord));
    path.push(Segment::Return);

    Svg {
        view_box: (days.start as Coord, 0, days.end as Coord, H as Coord),
        paths: vec![path],
    }
}
