use crate::{
    arrow::{
        block::delta::{
            single_column_storage::{
                SingleColumnStorage, SingleColumnStorageArrowValueCapacityHint,
            },
            BlockDelta, BlockStorage,
        },
        types::BuilderMutationOrderHint,
        types::{ArrowReadableValue, ArrowWriteableKey, ArrowWriteableValue},
    },
    key::KeyWrapper,
};
use arrow::{
    array::{Array, StringArray, StringBuilder},
    datatypes::Field,
    util::bit_util,
};
use std::sync::Arc;

impl ArrowWriteableValue for String {
    type ReadableValue<'referred_data> = &'referred_data str;
    type ArrowBuilder = StringBuilder;
    type ArrowCapacityHint = SingleColumnStorageArrowValueCapacityHint;
    type PreparedValue = String;

    fn offset_size(item_count: usize) -> usize {
        bit_util::round_upto_multiple_of_64((item_count + 1) * 4)
    }

    fn validity_size(_item_count: usize) -> usize {
        0 // We don't support None values for StringArray
    }

    fn add(prefix: &str, key: KeyWrapper, value: Self, delta: &BlockStorage) {
        match &delta {
            BlockStorage::String(builder) => builder.add(prefix, key, value),
            _ => panic!("Invalid builder type"),
        }
    }

    fn delete(prefix: &str, key: KeyWrapper, delta: &BlockDelta) {
        match &delta.builder {
            BlockStorage::String(builder) => builder.delete(prefix, key),
            _ => panic!("Invalid builder type"),
        }
    }

    fn get_delta_builder(mutation_ordering_hint: BuilderMutationOrderHint) -> BlockStorage {
        BlockStorage::String(SingleColumnStorage::new(mutation_ordering_hint))
    }

    fn get_arrow_builder(capacity_hint: Self::ArrowCapacityHint) -> Self::ArrowBuilder {
        StringBuilder::with_capacity(capacity_hint.item_count, capacity_hint.byte_size)
    }

    fn prepare(value: Self) -> Self::PreparedValue {
        value
    }

    fn append(value: Self::PreparedValue, builder: &mut Self::ArrowBuilder) {
        builder.append_value(value);
    }

    fn finish(mut builder: Self::ArrowBuilder) -> (arrow::datatypes::Field, Arc<dyn Array>) {
        let value_field = Field::new("value", arrow::datatypes::DataType::Utf8, false);
        let value_arr = builder.finish();
        let value_arr = (&value_arr as &dyn Array).slice(0, value_arr.len());
        (value_field, value_arr)
    }
}

impl<'referred_data> ArrowReadableValue<'referred_data> for &'referred_data str {
    fn get(array: &'referred_data Arc<dyn Array>, index: usize) -> &'referred_data str {
        let array = array.as_any().downcast_ref::<StringArray>().unwrap();
        array.value(index)
    }
    fn add_to_delta<K: ArrowWriteableKey>(
        prefix: &str,
        key: K,
        value: Self,
        delta: &mut BlockStorage,
    ) {
        String::add(prefix, key.into(), value.to_string(), delta);
    }
}
