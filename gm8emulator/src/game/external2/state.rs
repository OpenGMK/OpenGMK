use serde::{Deserialize, Serialize};
use super::dll;

#[derive(Clone, Serialize, Deserialize)]
pub enum State {
    DummyExternal {
        dll: String,
        symbol: String,
        dummy: dll::Value,
        argc: usize,
    },
    NormalExternal {
        dll: String,
        symbol: String,
        call_conv: dll::CallConv,
        type_args: Vec<dll::ValueType>,
        type_return: dll::ValueType,
    },
}
