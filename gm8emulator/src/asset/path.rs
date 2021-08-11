use crate::{gml, math::Real};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Path {
    pub name: gml::String,
    pub points: Vec<Point>,
    pub control_nodes: Vec<ControlNode>,
    pub length: Real,
    pub curve: bool,
    pub closed: bool,
    pub precision: i32,
    pub start: Point,
    pub end: Point,
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: Real,
    pub y: Real,
    pub speed: Real,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ControlNode {
    pub point: Point,
    pub distance: Real,
}

impl Path {
    /// Generates a new set of control nodes for this Path and updates its start, end and length
    pub fn update(&mut self) {
        self.control_nodes.clear(); // since you can dynamically add path points...
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
                        let point2 = self.points[(i + 2) % self.points.len()];
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
            if self.closed {
                if let Some(&first_point) = self.points.first() {
                    self.push_control_node(first_point);
                }
            }
        }
    }

    fn push_control_node(&mut self, point: Point) {
        let distance = match self.control_nodes.last() {
            Some(prev_node) => Real::from(point.distance(&prev_node.point)) + prev_node.distance,
            None => Real::from(0.0),
        };
        if self.control_nodes.len() == 0 {
            self.start = point;
        }
        self.end = point;
        self.control_nodes.push(ControlNode { point, distance });
        self.length = distance;
    }

    fn generate_smooth(&mut self, precision: i32, point1: Point, point2: Point, point3: Point) {
        if precision > 0 {
            let point_avg = Point {
                x: (point1.x + point2.x + point2.x + point3.x) / Real::from(4.0),
                y: (point1.y + point2.y + point2.y + point3.y) / Real::from(4.0),
                speed: (point1.speed + point2.speed + point2.speed + point3.speed) / Real::from(4.0),
            };

            if point1.distance(&point2) > Real::from(4.0) {
                self.generate_smooth(precision - 1, point1, point1.halfway_between(&point2), point_avg);
            }

            self.push_control_node(point_avg);

            if point2.distance(&point3) > Real::from(4.0) {
                self.generate_smooth(precision - 1, point_avg, point2.halfway_between(&point3), point3);
            }
        }
    }

    /// Returns the center of the Path
    pub fn center(&self) -> (Real, Real) {
        // GM defaults
        let mut left = Real::from(100000000);
        let mut right = Real::from(-100000000);
        let mut top = Real::from(100000000);
        let mut bottom = Real::from(-100000000);
        for point in &self.points {
            if point.x < left {
                left = point.x;
            }
            if point.x > right {
                right = point.x;
            }
            if point.y < top {
                top = point.y;
            }
            if point.y > bottom {
                bottom = point.y;
            }
        }
        ((left + right) / Real::from(2), (top + bottom) / Real::from(2))
    }

    /// Returns a Point on the path at the given offset, where 0 is the beginning and 1 is the end
    pub fn get_point(&self, offset: Real) -> Point {
        match &*self.control_nodes {
            [] => Point { x: Real::from(0.0), y: Real::from(0.0), speed: Real::from(0.0) },
            [node] => node.point,
            nodes => {
                let distance = offset * self.length;
                if distance <= Real::from(0.0) {
                    // We're before or at the first control node, so just return that
                    self.start
                } else {
                    for node_pair in nodes.windows(2) {
                        if distance >= node_pair[0].distance && distance <= node_pair[1].distance {
                            // We're between these two nodes, so lerp between them and return
                            let lerp =
                                (distance - node_pair[0].distance) / (node_pair[1].distance - node_pair[0].distance);
                            return Point {
                                x: (node_pair[1].point.x - node_pair[0].point.x) * lerp + node_pair[0].point.x,
                                y: (node_pair[1].point.y - node_pair[0].point.y) * lerp + node_pair[0].point.y,
                                speed: (node_pair[1].point.speed - node_pair[0].point.speed) * lerp
                                    + node_pair[0].point.speed,
                            }
                        }
                    }
                    // We must be after the final control node, so return that
                    self.end
                }
            },
        }
    }
}

impl Point {
    // Gives the distance in pixels between this and the provided point
    pub fn distance(&self, other: &Self) -> Real {
        let xdist = other.x - self.x;
        let ydist = other.y - self.y;
        (xdist * xdist + ydist * ydist).sqrt()
    }

    // Creates a point which is halfway between this and another point
    pub fn halfway_between(&self, other: &Self) -> Self {
        Self {
            x: (self.x + other.x) / Real::from(2.0),
            y: (self.y + other.y) / Real::from(2.0),
            speed: (self.speed + other.speed) / Real::from(2.0),
        }
    }
}
