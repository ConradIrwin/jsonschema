use crate::compilation::context::CompilationContext;
use crate::compilation::options::CustomKeywordDefinition;
use crate::keywords::CompilationResult;
use crate::paths::{InstancePath, JSONPointer, PathChunk};
use crate::validator::Validate;
use crate::ErrorIterator;
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

pub(crate) type CustomValidateFn =
    fn(&Value, JSONPointer, Arc<Value>, JSONPointer) -> ErrorIterator;
pub(crate) type CustomIsValidFn = fn(&Value, &Value) -> bool;

/// Custom keyword validation implemented by user provided validation functions.
pub(crate) struct CustomKeywordValidator {
    schema: Arc<Value>,
    schema_path: JSONPointer,
    validate: CustomValidateFn,
    is_valid: CustomIsValidFn,
}

impl Display for CustomKeywordValidator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Validate for CustomKeywordValidator {
    fn validate<'instance>(
        &self,
        instance: &'instance Value,
        instance_path: &InstancePath,
    ) -> ErrorIterator<'instance> {
        (self.validate)(
            instance,
            instance_path.into(),
            self.schema.clone(),
            self.schema_path.clone(),
        )
    }

    fn is_valid(&self, instance: &Value) -> bool {
        (self.is_valid)(instance, &self.schema)
    }
}

pub(crate) fn compile_custom_keyword_validator<'a>(
    context: &CompilationContext,
    keyword: impl Into<PathChunk>,
    keyword_definition: &CustomKeywordDefinition,
    schema: Value,
) -> CompilationResult<'a> {
    let schema_path = context.as_pointer_with(keyword);
    match keyword_definition {
        CustomKeywordDefinition::Validator { validate, is_valid } => {
            Ok(Box::new(CustomKeywordValidator {
                schema: Arc::new(schema),
                schema_path,
                validate: *validate,
                is_valid: *is_valid,
            }))
        }
    }
}
