use std::rc::Rc;

pub struct Path {
    pub name: Rc<str>,
    pub points: Vec<Point>,
    pub control_nodes: Vec<ControlNode>,
    pub length: f64,
    pub curve: bool,
    pub closed: bool,
    pub precision: i32,
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
}

#[derive(Copy, Clone)]
pub struct ControlNode {
    pub point: Point,
    pub distance: f64,
}

impl Path {
    /// Generates a new set of control nodes for this Path and updates its length
    pub fn update(&mut self) {
        if self.curve {
            if let (Some(&first_point), Some(&last_point)) = (self.points.first(), self.points.last()) {
                if !self.closed {
                    self.push_control_node(first_point);
                }

                let loop_count = if self.closed { Some(self.points.len()) } else { self.points.len().checked_sub(2) };
                if let Some(loop_count) = loop_count {
                    for i in 0..loop_count {
                        let point0 = self.points[i % self.points.len()];
                        let point1 = self.points[(i + 1) % self.points.len()];
                        let point2 = self.points[(i + 2) & self.points.len()];
                        self.generate_smooth(
                            self.precision,
                            point0.halfway_between(&point1),
                            point1,
                            point1.halfway_between(&point2),
                        );
                    }
                }
                if self.closed {
                    match self.control_nodes.first() {
                        Some(&first_node) => self.push_control_node(first_node.point),
                        None => self.push_control_node(first_point),
                    }
                } else {
                    self.push_control_node(last_point);
                }
            }
        } else {
            for i in 0..self.points.len() {
                self.push_control_node(self.points[i]);
            }
        }
    }

    fn push_control_node(&mut self, point: Point) {
        let distance = match self.control_nodes.last() {
            Some(prev_node) => point.distance(&prev_node.point) + prev_node.distance,
            None => 0.0,
        };
        self.control_nodes.push(ControlNode { point, distance });
        self.length = distance;
    }

    fn generate_smooth(&mut self, precision: i32, point1: Point, point2: Point, point3: Point) {
        if precision > 0 {
            let point_avg = Point {
                x: (point1.x + point2.x + point2.x + point3.x) / 4.0,
                y: (point1.y + point2.y + point2.y + point3.y) / 4.0,
                speed: (point1.speed + point2.speed + point2.speed + point3.speed) / 4.0,
            };

            if point1.distance(&point2) > 4.0 {
                self.generate_smooth(precision - 1, point1, point1.halfway_between(&point2), point_avg);
            }

            self.push_control_node(point_avg);

            if point2.distance(&point3) > 4.0 {
                self.generate_smooth(precision - 1, point_avg, point2.halfway_between(&point3), point3);
            }
        }
    }
}

impl Point {
    // Gives the distance in pixels between this and the provided point
    pub fn distance(&self, other: &Self) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }

    // Creates a point which is halfway between this and another point
    pub fn halfway_between(&self, other: &Self) -> Self {
        Self { x: (self.x + other.x) / 2.0, y: (self.y + other.y) / 2.0, speed: (self.speed + other.speed) / 2.0 }
    }
}
