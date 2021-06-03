use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Path {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,

    pub connection: Connection,
    pub closed: bool,
    pub precision: u32,
    pub editor_reference: EditorReference,
    pub points: Vec<Point>,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Connection {
    Straight = 0,
    Smooth = 1,
}

impl From<u32> for Connection {
    fn from(n: u32) -> Self {
        match n {
            0 => Self::Straight,
            1 => Self::Smooth,
            _ => Self::Smooth,
        }
    }
}

pub struct EditorReference {
    pub room_bg: i32,
    pub grid_snap: (u32, u32),
}

pub struct Point {
    pub position: (f64, f64),
    pub speed: f64,
}

impl Asset for Path {
    #[inline]
    fn name(&self) -> &[u8] {
        self.name.0.as_slice()
    }

    #[inline]
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    #[inline]
    fn version(&self) -> Version {
        self.version
    }

    fn from_gmk<R: io::Read>(&self, mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, true)
    }

    fn to_gmk<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, true)
    }

    fn from_exe<R: io::Read>(&self, mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, false)
    }

    fn to_exe<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, false)
    }
}

impl Path {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "path", Gm530)?;

        let connection = Connection::from(reader.read_u32::<LE>()?);
        let closed = reader.read_u32::<LE>()? != 0;
        let precision = reader.read_u32::<LE>()?;
        let editor_reference = if is_gmk {
            EditorReference::read(&mut reader)?
        } else {
            EditorReference::default()
        };
        let points = (0..reader.read_u32::<LE>()?)
            .map(|_| Point::read(&mut reader))
            .collect::<io::Result<Vec<Point>>>()?;
        Ok(Self { name, timestamp, version, connection, closed, precision, editor_reference, points })
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm530);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;

        writer.write_u32::<LE>(self.connection as u32)?;
        writer.write_u32::<LE>(self.closed.into())?;
        writer.write_u32::<LE>(self.precision)?;
        if is_gmk {
            self.editor_reference.write(&mut writer)?;
        }
        assert!(self.points.len() <= u32::max_value() as usize);
        writer.write_u32::<LE>(self.points.len() as u32)?;
        for point in &self.points {
            point.write(&mut writer)?;
        }
        Ok(())
    }
}

impl EditorReference {
    fn read(reader: &mut dyn io::Read) -> io::Result<Self> {
        let room_bg = reader.read_i32::<LE>()?;
        let grid_snap = (reader.read_u32::<LE>()?, reader.read_u32::<LE>()?);
        Ok(Self { room_bg, grid_snap })
    }

    fn write(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        writer.write_i32::<LE>(self.room_bg)?;
        writer.write_u32::<LE>(self.grid_snap.0)?;
        writer.write_u32::<LE>(self.grid_snap.1)
    }
}

impl Default for EditorReference {
    fn default() -> Self {
        Self {
            room_bg: -1,
            grid_snap: (16, 16),
        }
    }
}

impl Point {
    fn read(reader: &mut dyn io::Read) -> io::Result<Self> {
        let position = (reader.read_f64::<LE>()?, reader.read_f64::<LE>()?);
        let speed = reader.read_f64::<LE>()?;
        Ok(Self { position, speed })
    }

    fn write(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        writer.write_f64::<LE>(self.position.0)?;
        writer.write_f64::<LE>(self.position.1)?;
        writer.write_f64::<LE>(self.speed)
    }
}
