#[derive(Debug)]
pub struct Sound {
    /// Asset name
    pub name: String,

    /// Any of: normal, background, 3d, use multimedia player
    /// I should make this an enum eventually. TODO
    pub kind: u32,

    pub file_type: String,
    pub file_name: String,

    /// This is optional because the associated data doesn't need to exist. Fantastic.
    pub file_data: Option<Box<[u8]>>,

    /// Volume - Between 0 and 1, although the editor only allows as low as 0.3
    pub volume: f64,

    /// 3D Pan - Between -1 and 1 (L <-> R)
    pub pan: f64,

    /// TODO: I have no idea what this does.
    pub preload: bool,

    pub version: u32,
}
