use crate::paths::{Location, LocationSegment};

use crate::{
    error::{error, ErrorIterator, ValidationError},
    keywords::CompilationResult,
    validator::Validate,
};
use referencing::List;
use serde_json::Value;

pub(crate) struct FalseValidator {
    location: Location,
}
impl FalseValidator {
    #[inline]
    pub(crate) fn compile<'a>(location: Location) -> CompilationResult<'a> {
        Ok(Box::new(FalseValidator { location }))
    }
}
impl Validate for FalseValidator {
    fn is_valid(&self, _: &Value) -> bool {
        false
    }

    fn validate<'i>(
        &self,
        instance: &'i Value,
        location: List<LocationSegment<'i>>,
    ) -> ErrorIterator<'i> {
        error(ValidationError::false_schema(
            self.location.clone(),
            location.into(),
            instance,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_util;
    use serde_json::json;

    #[test]
    fn location() {
        tests_util::assert_schema_location(&json!(false), &json!(1), "")
    }
}
