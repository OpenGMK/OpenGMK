use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read};

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
    fn deserialize_exe(mut reader: impl Read, _version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        let version = reader.read_u32::<LE>()?;
        if strict {
            assert_ver(version, VERSION)?;
        }

        let connection = ConnectionKind::from(reader.read_u32::<LE>()?);

        let closed = reader.read_u32::<LE>()? != 0;
        let precision = reader.read_u32::<LE>()?;

        let point_count = reader.read_u32::<LE>()? as usize;
        let points = (0..point_count)
            .map(|_| {
                Ok(Point { x: reader.read_f64::<LE>()?, y: reader.read_f64::<LE>()?, speed: reader.read_f64::<LE>()? })
            })
            .collect::<io::Result<_>>()?;

        Ok(Path { name, connection, precision, closed, points })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, _version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_u32::<LE>(self.connection as u32)?;
        writer.write_u32::<LE>(self.closed.into())?;
        writer.write_u32::<LE>(self.precision)?;
        // TODO: add debug assertions for these lengths everywhere, for real
        writer.write_u32::<LE>(self.points.len() as u32)?;
        for point in self.points.iter() {
            writer.write_f64::<LE>(point.x)?;
            writer.write_f64::<LE>(point.y)?;
            writer.write_f64::<LE>(point.speed)?;
        }
        Ok(())
    }
}
