use boa_engine::value::TryFromJs;
use boa_engine::{js_string, Context, JsNativeError, JsResult, JsValue};
use boa_macros::{Finalize, Trace};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Trace, Finalize)]
pub struct JsVec<T>(pub Vec<T>);

impl<T: TryFromJs> TryFromJs for JsVec<T> {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        let object = match value {
            JsValue::Object(o) => o,
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("cannot convert value to a Vec")
                    .into())
            }
        };

        let length = object
            .get(js_string!("length"), context)?
            .to_length(context)?;
        let length = match usize::try_from(length) {
            Ok(length) => length,
            Err(e) => {
                return Err(JsNativeError::typ()
                    .with_message(format!("could not convert length to usize: {e}"))
                    .into());
            }
        };
        let mut vec = Vec::with_capacity(length);
        for i in 0..length {
            let value = object.get(i, context)?;
            vec.push(T::try_from_js(&value, context)?);
        }

        Ok(Self(vec))
    }
}
