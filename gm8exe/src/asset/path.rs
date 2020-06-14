use crate::{
    asset::{assert_ver, Asset, AssetDataError, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};

use minio::{ReadPrimitives, WritePrimitives};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: u32 = 530;

pub struct Path {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The kind of interpolation used between the path points.
    pub connection: ConnectionKind,

    /// This number represents the level of precision when using smooth curves.
    /// Value can be 1 through 8, 1 being the most coarse and 8 being the curviest.
    pub precision: u32,

    /// Indicates whether the path points represent a closed shape.
    pub closed: bool,

    /// The list of points that connect to create this Path.
    pub points: Vec<Point>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ConnectionKind {
    /// Normal, linear point-to-point path.
    StraightLine = 0,

    /// Interpolated smooth curves.
    SmoothCurve = 1,
}

pub struct Point {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
}

impl From<u32> for ConnectionKind {
    fn from(n: u32) -> ConnectionKind {
        match n {
            0 => ConnectionKind::StraightLine,
            1 => ConnectionKind::SmoothCurve,

            _ => ConnectionKind::SmoothCurve,
        }
    }
}

impl Asset for Path {
    fn deserialize<B>(bytes: B, strict: bool, _version: GameVersion) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized,
    {
        let mut reader = io::Cursor::new(bytes.as_ref());
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32_le()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let connection = ConnectionKind::from(reader.read_u32_le()?);

        let closed = reader.read_u32_le()? != 0;
        let precision = reader.read_u32_le()?;

        let point_count = reader.read_u32_le()? as usize;
        let mut points = Vec::with_capacity(point_count);
        for _ in 0..point_count {
            points.push(Point { x: reader.read_f64_le()?, y: reader.read_f64_le()?, speed: reader.read_f64_le()? });
        }

        Ok(Path { name, connection, precision, closed, points })
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION)?;
        result += writer.write_u32_le(self.connection as u32)?;
        result += writer.write_u32_le(self.closed as u32)?;
        result += writer.write_u32_le(self.precision as u32)?;
        result += writer.write_u32_le(self.points.len() as u32)?;
        for point in self.points.iter() {
            result += writer.write_f64_le(point.x)?;
            result += writer.write_f64_le(point.y)?;
            result += writer.write_f64_le(point.speed)?;
        }

        Ok(result)
    }
}
