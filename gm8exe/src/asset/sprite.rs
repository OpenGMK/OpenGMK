use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::{
    convert::TryInto,
    io::{self, Seek, SeekFrom},
};
use crate::asset::ReadChunk;

pub const VERSION: u32 = 800;
pub const VERSION_COLLISION: u32 = 800;
pub const VERSION_FRAME: u32 = 800;

pub struct Sprite {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The origin within the sprite.
    pub origin_x: i32,

    /// The origin within the sprite.
    pub origin_y: i32,

    /// The raw RGBA pixeldata for each frame.
    pub frames: Vec<Frame>,

    /// The collider associated with one or each frame.
    /// If `per_frame_colliders` is false, this contains 1 map.
    pub colliders: Vec<CollisionMap>,

    /// Whether each individual frame has its own collision map.
    pub per_frame_colliders: bool,
}

pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub data: Box<[u8]>,
}

pub struct CollisionMap {
    // width of the boolean map
    pub width: u32,

    // height of the boolean map
    pub height: u32,

    // left-most x coordinate in the map which has collision
    pub bbox_left: u32,

    // right-most x coordinate in the map which has collision
    pub bbox_right: u32,

    // top-most (lowest value) y coordinate in the map which has collision
    pub bbox_top: u32,

    // bottom-most (highest value) y coordinate in the map which has collision
    pub bbox_bottom: u32,

    // Map of collision data - boolean for whether each pixel has collision
    pub data: Box<[bool]>,
}

impl Asset for Sprite {
    fn deserialize_exe(mut reader: impl io::Read + io::Seek, version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32::<LE>()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let origin_x = reader.read_i32::<LE>()?;
        let origin_y = reader.read_i32::<LE>()?;
        let frame_count = reader.read_u32::<LE>()?;
        let (frames, colliders, per_frame_colliders) = if frame_count != 0 {
            let mut frames = Vec::with_capacity(frame_count as usize);
            for _ in 0..frame_count {
                if strict {
                    let version = reader.read_u32::<LE>()?;
                    assert_ver(version, VERSION_FRAME)?;
                } else {
                    reader.seek(SeekFrom::Current(4))?;
                }

                let frame_width = reader.read_u32::<LE>()?;
                let frame_height = reader.read_u32::<LE>()?;

                let len = reader.read_u32::<LE>()? as usize;
                let data = reader.read_chunk(len)?.into_boxed_slice();

                frames.push(Frame { width: frame_width, height: frame_height, data });
            }

            fn read_collision<T>(reader: &mut io::Cursor<T>, strict: bool) -> Result<CollisionMap, Error>
            where
                T: AsRef<[u8]>,
            {
                if strict {
                    let version = reader.read_u32::<LE>()?;
                    assert_ver(version, VERSION_COLLISION)?;
                } else {
                    reader.seek(SeekFrom::Current(4))?;
                }

                let width = reader.read_u32::<LE>()?;
                let height = reader.read_u32::<LE>()?;
                let bbox_left = reader.read_u32::<LE>()?;
                let bbox_right = reader.read_u32::<LE>()?;
                let bbox_bottom = reader.read_u32::<LE>()?;
                let bbox_top = reader.read_u32::<LE>()?;

                let mask_size = width as usize * height as usize;
                let pos = reader.position() as usize;
                reader.seek(SeekFrom::Current(4 * mask_size as i64))?;
                let mask: Vec<bool> = match reader.get_ref().as_ref().get(pos..pos + (4 * mask_size)) {
                    Some(b) => b
                        .chunks_exact(4)
                        .map(|ch| {
                            // until we get const generics we need to do this to get an exact array.
                            // panic is unreachable and is optimized out.
                            // TODO: you can use byteorder for this!! please!!!
                            u32::from_le_bytes(*<&[u8] as TryInto<&[u8; 4]>>::try_into(ch).unwrap()) != 0
                        })
                        .collect(),
                    None => return Err(Error::MalformedData),
                };

                Ok(CollisionMap {
                    width,
                    height,
                    bbox_left,
                    bbox_right,
                    bbox_bottom,
                    bbox_top,
                    data: mask.into_boxed_slice(),
                })
            }

            let mut colliders: Vec<CollisionMap>;
            let per_frame_colliders = reader.read_u32::<LE>()? != 0;
            if per_frame_colliders {
                colliders = Vec::with_capacity(frame_count as usize);
                for _ in 0..frame_count {
                    colliders.push(read_collision(&mut reader, strict)?);
                }
            } else {
                colliders = vec![read_collision(&mut reader, strict)?];
            }
            (frames, colliders, per_frame_colliders)
        } else {
            (Vec::new(), Vec::new(), false)
        };

        Ok(Sprite { name, origin_x, origin_y, frames, colliders, per_frame_colliders })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_i32::<LE>(self.origin_x)?;
        writer.write_i32::<LE>(self.origin_y)?;
        if !self.frames.is_empty() {
            writer.write_u32::<LE>(self.frames.len() as u32)?; // TODO: len as u32
            for frame in self.frames.iter() {
                writer.write_u32::<LE>(VERSION_FRAME)?;
                writer.write_u32::<LE>(frame.width)?;
                writer.write_u32::<LE>(frame.height)?;
                writer.write_u32::<LE>(frame.data.len() as u32)?;
                let pixeldata = frame.data.clone();
                writer.write_all(&pixeldata)?;
            }
            writer.write_u32::<LE>(self.per_frame_colliders as u32)?;
            for collider in self.colliders.iter() {
                writer.write_u32::<LE>(VERSION_COLLISION)?;
                writer.write_u32::<LE>(collider.width)?;
                writer.write_u32::<LE>(collider.height)?;
                writer.write_u32::<LE>(collider.bbox_left)?;
                writer.write_u32::<LE>(collider.bbox_right)?;
                writer.write_u32::<LE>(collider.bbox_bottom)?;
                writer.write_u32::<LE>(collider.bbox_top)?;
                for pixel in &*collider.data {
                    writer.write_u32::<LE>(*pixel as u32)?;
                }
            }
        } else {
            writer.write_u32::<LE>(0)?;
        }
        Ok(())
    }
}
