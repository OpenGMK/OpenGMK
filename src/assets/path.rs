use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::parser::ParserOptions;
use crate::types::Version;
use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 530;

#[derive(Copy, Clone, PartialEq)]
pub enum ConnectionKind {
    /// Normal, linear point-to-point path.
    StraightLine = 0,

    /// Interpolated smooth curves.
    SmoothCurve = 1,
}

pub struct Path {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The kind of interpolation used between the path points.
    pub connection: ConnectionKind,

    /// This number represents the level of precision when using smooth curves.
    /// Value can be 1 through 8, 1 being the most precise and 8 being the curviest.
    pub precision: u32,

    /// Indicates whether the path points represent a closed shape.
    pub closed: bool,

    /// The list of points that connect to create this Path.
    pub points: Vec<Point>,
}

pub struct Point {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
}

impl Path {
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION as u32)?;
        result += writer.write_u32_le(self.connection as u32)?;
        result += writer.write_u32_le(if self.closed { 1 } else { 0 })?;
        result += writer.write_u32_le(self.precision as u32)?;
        result += writer.write_u32_le(self.points.len() as u32)?;
        for point in self.points.iter() {
            result += writer.write_f64_le(point.x)?;
            result += writer.write_f64_le(point.y)?;
            result += writer.write_f64_le(point.speed)?;
        }

        Ok(result)
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Path>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if options.strict {
            let version = reader.read_u32_le()? as Version;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let connection = if reader.read_u32_le()? == 0 {
            ConnectionKind::StraightLine
        } else {
            ConnectionKind::SmoothCurve
        };

        let closed = reader.read_u32_le()? != 0;
        let precision = reader.read_u32_le()?;

        let point_count = reader.read_u32_le()?;
        let mut points = Vec::new();
        for _ in 0..point_count {
            points.push(Point {
                x: reader.read_f64_le()?,
                y: reader.read_f64_le()?,
                speed: reader.read_f64_le()?,
            });
        }

        Ok(Path {
            name,
            connection,
            precision,
            closed,
            points,
        })
    }
}
