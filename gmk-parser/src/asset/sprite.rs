use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Sprite {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,

    pub origin: (i32, i32),
    pub frames: Vec<Frame>,
    pub colliders: Vec<Collider>,

    pub per_frame_colliders: bool,
}

pub struct BoundingBox {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

pub enum Collider {
    Normal(()),
    Baked(BakedCollider),
}

pub struct BakedCollider {
    pub version: Version,
    pub size: (u32, u32),
    pub bbox: BoundingBox,
    pub data: Vec<bool>,
}

pub struct Frame {
    pub version: Version,
    pub size: (u32, u32),
    pub data: Vec<u8>,
}

impl Asset for Sprite {
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

impl Sprite {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "sprite", Gm800)?;

        let origin = (
            reader.read_i32::<LE>()?,
            reader.read_i32::<LE>()?,
        );

        let frames = (0..reader.read_u32::<LE>()? as usize).map(|_| {
            let version = read_version!(reader, name, is_gmk, "frame in sprite", Gm800)?;
            let size = (
                reader.read_u32::<LE>()?,
                reader.read_u32::<LE>()?,
            );
            let data = if size.0 > 0 && size.1 > 0 {
                let data_len = reader.read_u32::<LE>()? as usize;
                let mut data = Vec::with_capacity(data_len);
                unsafe { data.set_len(data_len) };
                reader.read_exact(data.as_mut_slice())?;
                data
            } else {
                Vec::new()
            };
            Ok(Frame { version, size, data })
        }).collect::<io::Result<Vec<Frame>>>()?;

        let (colliders, per_frame_colliders) = if is_gmk {
            todo!()
        } else {
            let per_frame_colliders = reader.read_u32::<LE>()? != 0;
            let colliders = (0..if per_frame_colliders { frames.len() } else { 1 }).map(|_| {
                let version = read_version!(reader, name, is_gmk, "collider in sprite", Gm800)?;
                let size = (
                    reader.read_u32::<LE>()?,
                    reader.read_u32::<LE>()?,
                );
                let bbox = BoundingBox::read(&mut reader)?;
                let data = (0..reader.read_u32::<LE>()? as usize)
                    .map(|_| reader.read_u32::<LE>().map(|x| x != 0))
                    .collect::<io::Result<Vec<bool>>>()?;
                Ok(Collider::Baked(BakedCollider { version, size, bbox, data }))
            }).collect::<io::Result<Vec<Collider>>>()?;
            (colliders, per_frame_colliders)
        };

        Ok(Self { name, timestamp, version, origin, frames, colliders, per_frame_colliders })
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm800);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;

        writer.write_i32::<LE>(self.origin.0)?;
        writer.write_i32::<LE>(self.origin.1)?;

        assert!(self.frames.len() <= u32::max_value() as usize);
        writer.write_u32::<LE>(self.frames.len() as u32)?;
        for frame in &self.frames {
            assert_eq!(frame.version, Version::Gm800);
            writer.write_u32::<LE>(frame.version as u32)?;
            writer.write_u32::<LE>(frame.size.0)?;
            writer.write_u32::<LE>(frame.size.1)?;
            if frame.size.0 > 0 && frame.size.1 > 0 {
                assert!(frame.data.len() <= u32::max_value() as usize);
                writer.write_u32::<LE>(frame.data.len() as u32)?;
                writer.write_all(frame.data.as_slice())?;
            }
        }

        if is_gmk {
            todo!()
        } else {
            writer.write_u32::<LE>(self.per_frame_colliders as u32)?;
            for collider in &self.colliders {
                match collider {
                    Collider::Normal(_) => todo!(),
                    Collider::Baked(map) => {
                        writer.write_u32::<LE>(map.version as u32)?;
                        writer.write_u32::<LE>(map.size.0)?;
                        writer.write_u32::<LE>(map.size.1)?;
                        map.bbox.write(&mut writer)?;
                        for pixel in &*map.data {
                            writer.write_u32::<LE>(u32::from(*pixel))?;
                        }
                    },
                }
            }
        }

        todo!()
    }
}

impl BoundingBox {
    fn read(reader: &mut dyn io::Read) -> io::Result<Self> {
        let left = reader.read_u32::<LE>()?;
        let right = reader.read_u32::<LE>()?;
        let bottom = reader.read_u32::<LE>()?;
        let top = reader.read_u32::<LE>()?;
        Ok(Self { left, right, bottom, top })
    }

    fn write(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        writer.write_u32::<LE>(self.left)?;
        writer.write_u32::<LE>(self.right)?;
        writer.write_u32::<LE>(self.bottom)?;
        writer.write_u32::<LE>(self.top)?;
        Ok(())
    }
}
