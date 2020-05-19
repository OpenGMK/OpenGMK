#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct Real(f64);

impl From<f64> for Real {
    fn from(f: f64) -> Self {
        Real(f)
    }
}