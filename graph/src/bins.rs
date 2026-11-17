use std::ops::Range;

pub const INTERVALS: [Range<f64>; 5] = [
    (0.0..60.0),
    (60.0..300.0),
    (300.0..600.0),
    (600.0..1800.0),
    (1800.0..f64::INFINITY)
];

pub const CATEGORIES: [&'static str; 2] = [
    "newfound",
    "refound"
];
