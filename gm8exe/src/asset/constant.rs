use crate::asset::PascalString;

pub struct Constant {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The GML to be evaluated into a constant value at loading time.
    ///
    /// The official runner cannot invoke user defined Script assets in this, will execute invalid memory.
    pub expression: PascalString,
}
