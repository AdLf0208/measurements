// Measurements for Horizon — the tape-measure model (pure).
//
// Ported from Measurements 3.0.3 (Mrbysco/Measurements, MIT) for Minecraft
// Java 1.21.1 NeoForge. Upstream source was read for behaviour only; this is a
// clean-room Rust rewrite.
//
// Upstream keeps a client-only list of measurement boxes: the first right
// click starts a box at the clicked block, the second finishes it, a further
// click starts the next box, and shift-right-click removes the latest box
// (`BoxHandler.java:15-37`). The box is drawn with one integer label per axis
// (`MeasurementBox.java:118-173`). The port keeps the same two-click model on
// the server and reports the Euclidean distance plus the per-axis deltas as an
// action-bar line (a recorded deviation: the pin cannot draw the client box
// headlessly, and upstream shows no distance).

/// The one item that starts a measurement.
pub const TAPE_MEASURE: &str = "measurements:tape_measure";

/// A block position in the measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub const fn from_array([x, y, z]: [i32; 3]) -> Self {
        Self::new(x, y, z)
    }
}

/// The absolute difference between two points on each axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delta {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Delta {
    pub fn between(a: Point, b: Point) -> Self {
        Self {
            x: a.x.abs_diff(b.x),
            y: a.y.abs_diff(b.y),
            z: a.z.abs_diff(b.z),
        }
    }

    /// The Euclidean length of the delta, in blocks.
    pub fn distance(self) -> f64 {
        let x = f64::from(self.x);
        let y = f64::from(self.y);
        let z = f64::from(self.z);
        (x * x + y * y + z * z).sqrt()
    }
}

/// One measurement: a start point and, once the second click arrives, an end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measurement {
    pub start: Point,
    pub end: Option<Point>,
}

impl Measurement {
    pub const fn new(start: Point) -> Self {
        Self { start, end: None }
    }

    pub const fn is_finished(self) -> bool {
        self.end.is_some()
    }
}

/// What a click did.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// The click placed point A.
    Started { at: Point },
    /// The click placed point B and finished the measurement.
    Completed {
        start: Point,
        end: Point,
        delta: Delta,
        distance: f64,
    },
    /// A shift-click removed the latest measurement.
    Undone { remaining: usize },
    /// A shift-click found nothing to remove.
    NothingToUndo,
}

/// One player's measurements. Upstream keeps one client-side list; the port
/// keys it by player because it runs on the server (recorded deviation).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Session {
    measurements: Vec<Measurement>,
}

impl Session {
    pub const fn new() -> Self {
        Self {
            measurements: Vec::new(),
        }
    }

    /// The measurements taken so far, oldest first. Test-only: the game path
    /// only needs the outcome of a click.
    #[cfg(test)]
    pub fn measurements(&self) -> &[Measurement] {
        &self.measurements
    }

    /// Applies one right click. A plain click completes the latest unfinished
    /// measurement, or starts the next one. A shift-click removes the latest
    /// measurement (upstream's `undo`).
    pub fn click(&mut self, pos: Point, sneaking: bool) -> Outcome {
        if sneaking {
            return match self.measurements.pop() {
                Some(_) => Outcome::Undone {
                    remaining: self.measurements.len(),
                },
                None => Outcome::NothingToUndo,
            };
        }
        if let Some(last) = self.measurements.last_mut()
            && !last.is_finished()
        {
            last.end = Some(pos);
            let start = last.start;
            let delta = Delta::between(start, pos);
            return Outcome::Completed {
                start,
                end: pos,
                delta,
                distance: delta.distance(),
            };
        }
        self.measurements.push(Measurement::new(pos));
        Outcome::Started { at: pos }
    }

    /// Drops every measurement (the tape is no longer held, or the dimension
    /// changed). Returns whether anything was dropped.
    pub fn clear(&mut self) -> bool {
        let had = !self.measurements.is_empty();
        self.measurements.clear();
        had
    }
}

/// Formats the Euclidean distance with one decimal place.
pub fn format_distance(distance: f64) -> String {
    format!("{distance:.1}")
}

/// The action-bar line for an outcome.
pub fn readout(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Started { at } => format!("Point A: {} {} {}", at.x, at.y, at.z),
        Outcome::Completed {
            delta, distance, ..
        } => format!(
            "Distance: {} blocks (X: {}, Y: {}, Z: {})",
            format_distance(*distance),
            delta.x,
            delta.y,
            delta.z
        ),
        Outcome::Undone { remaining: 0 } => "Measurement cleared".to_owned(),
        Outcome::Undone { remaining } => format!("Measurement undone ({remaining} left)"),
        Outcome::NothingToUndo => "No measurement to clear".to_owned(),
    }
}

/// Whether an item id is the tape measure.
pub fn is_tape_measure(item: &str) -> bool {
    item == TAPE_MEASURE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: i32, y: i32, z: i32) -> Point {
        Point::new(x, y, z)
    }

    #[test]
    fn the_first_click_starts_a_measurement() {
        let mut session = Session::new();
        let outcome = session.click(at(0, 64, 0), false);
        assert_eq!(outcome, Outcome::Started { at: at(0, 64, 0) });
        assert_eq!(session.measurements().len(), 1);
        assert!(!session.measurements()[0].is_finished());
        assert_eq!(readout(&outcome), "Point A: 0 64 0");
    }

    #[test]
    fn the_second_click_completes_and_reports_the_distance() {
        let mut session = Session::new();
        session.click(at(0, 64, 0), false);
        let outcome = session.click(at(5, 64, 0), false);
        assert_eq!(
            outcome,
            Outcome::Completed {
                start: at(0, 64, 0),
                end: at(5, 64, 0),
                delta: Delta { x: 5, y: 0, z: 0 },
                distance: 5.0,
            }
        );
        assert!(session.measurements()[0].is_finished());
        assert_eq!(readout(&outcome), "Distance: 5.0 blocks (X: 5, Y: 0, Z: 0)");
    }

    #[test]
    fn another_axis_measures_its_own_delta() {
        let mut session = Session::new();
        session.click(at(0, 64, 0), false);
        let outcome = session.click(at(0, 64, 5), false);
        assert_eq!(readout(&outcome), "Distance: 5.0 blocks (X: 0, Y: 0, Z: 5)");
    }

    #[test]
    fn a_diagonal_distance_gets_one_decimal() {
        let mut session = Session::new();
        session.click(at(0, 0, 0), false);
        let outcome = session.click(at(5, 5, 0), false);
        assert_eq!(readout(&outcome), "Distance: 7.1 blocks (X: 5, Y: 5, Z: 0)");
        assert_eq!(format_distance(3.0_f64.hypot(4.0)), "5.0");
        assert_eq!(format_distance(0.0), "0.0");
    }

    #[test]
    fn negative_coordinates_use_absolute_deltas() {
        let mut session = Session::new();
        session.click(at(10, -5, -8), false);
        let outcome = session.click(at(-2, 7, 4), false);
        assert_eq!(
            outcome,
            Outcome::Completed {
                start: at(10, -5, -8),
                end: at(-2, 7, 4),
                delta: Delta {
                    x: 12,
                    y: 12,
                    z: 12
                },
                distance: 432.0_f64.sqrt(),
            }
        );
        assert_eq!(
            readout(&outcome),
            "Distance: 20.8 blocks (X: 12, Y: 12, Z: 12)"
        );
    }

    #[test]
    fn measuring_the_same_block_twice_is_zero() {
        let mut session = Session::new();
        session.click(at(3, 70, -4), false);
        let outcome = session.click(at(3, 70, -4), false);
        assert_eq!(readout(&outcome), "Distance: 0.0 blocks (X: 0, Y: 0, Z: 0)");
    }

    #[test]
    fn a_click_after_a_finished_measurement_starts_the_next_one() {
        let mut session = Session::new();
        session.click(at(0, 64, 0), false);
        session.click(at(5, 64, 0), false);
        let outcome = session.click(at(1, 65, 1), false);
        assert_eq!(outcome, Outcome::Started { at: at(1, 65, 1) });
        assert_eq!(session.measurements().len(), 2);
        assert!(!session.measurements()[1].is_finished());
    }

    #[test]
    fn shift_click_undoes_only_the_latest_measurement() {
        let mut session = Session::new();
        session.click(at(0, 64, 0), false);
        session.click(at(5, 64, 0), false);
        session.click(at(0, 64, 0), false);
        session.click(at(0, 64, 5), false);
        let outcome = session.click(at(9, 9, 9), true);
        assert_eq!(outcome, Outcome::Undone { remaining: 1 });
        assert_eq!(session.measurements().len(), 1);
        assert_eq!(session.measurements()[0].start, at(0, 64, 0));
        assert_eq!(session.measurements()[0].end, Some(at(5, 64, 0)));
    }

    #[test]
    fn a_second_shift_click_removes_the_rest() {
        let mut session = Session::new();
        session.click(at(0, 64, 0), false);
        session.click(at(5, 64, 0), false);
        assert_eq!(
            session.click(at(9, 9, 9), true),
            Outcome::Undone { remaining: 0 }
        );
        assert_eq!(session.click(at(9, 9, 9), true), Outcome::NothingToUndo);
        assert!(session.measurements().is_empty());
    }

    #[test]
    fn an_unfinished_measurement_can_be_undone() {
        let mut session = Session::new();
        session.click(at(0, 64, 0), false);
        assert_eq!(
            session.click(at(0, 64, 0), true),
            Outcome::Undone { remaining: 0 }
        );
        assert!(session.measurements().is_empty());
    }

    #[test]
    fn clearing_drops_every_measurement() {
        let mut session = Session::new();
        session.click(at(0, 64, 0), false);
        session.click(at(5, 64, 0), false);
        assert!(session.clear());
        assert!(session.measurements().is_empty());
        assert!(!session.clear());
    }

    #[test]
    fn only_the_tape_measure_starts_a_measurement() {
        assert!(is_tape_measure(TAPE_MEASURE));
        assert!(!is_tape_measure("minecraft:stick"));
        assert!(!is_tape_measure("measurements:other"));
    }

    #[test]
    fn a_started_readout_keeps_integer_coordinates() {
        let outcome = Outcome::Started {
            at: at(-12, 64, 103),
        };
        assert_eq!(readout(&outcome), "Point A: -12 64 103");
    }
}
