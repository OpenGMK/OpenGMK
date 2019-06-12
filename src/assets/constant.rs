pub struct Constant {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The source GML to be evaluated into a constant value at loading time.
    pub expression: String,
}
