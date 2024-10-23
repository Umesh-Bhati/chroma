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
    array::{Array, UInt32Array, UInt32Builder},
    datatypes::Field,
};
use std::sync::Arc;

impl ArrowWriteableValue for u32 {
    type ReadableValue<'referred_data> = u32;
    type ArrowBuilder = UInt32Builder;
    type ArrowCapacityHint = SingleColumnStorageArrowValueCapacityHint;
    type PreparedValue = u32;

    fn offset_size(_item_count: usize) -> usize {
        0
    }

    fn validity_size(_item_count: usize) -> usize {
        0 // We don't support None values for UInt32Array
    }

    fn add(prefix: &str, key: KeyWrapper, value: Self, delta: &BlockStorage) {
        match &delta {
            BlockStorage::UInt32(builder) => builder.add(prefix, key, value),
            _ => panic!("Invalid builder type: {:?}", &delta),
        }
    }

    fn delete(prefix: &str, key: KeyWrapper, delta: &BlockDelta) {
        match &delta.builder {
            BlockStorage::UInt32(builder) => builder.delete(prefix, key),
            _ => panic!("Invalid builder type: {:?}", &delta.builder),
        }
    }

    fn get_delta_builder(mutation_ordering_hint: BuilderMutationOrderHint) -> BlockStorage {
        BlockStorage::UInt32(SingleColumnStorage::new(mutation_ordering_hint))
    }

    fn get_arrow_builder(capacity_hint: Self::ArrowCapacityHint) -> Self::ArrowBuilder {
        UInt32Builder::with_capacity(capacity_hint.item_count)
    }

    fn prepare(value: Self) -> Self::PreparedValue {
        value
    }

    fn append(value: Self::PreparedValue, builder: &mut Self::ArrowBuilder) {
        builder.append_value(value);
    }

    fn finish(mut builder: Self::ArrowBuilder) -> (arrow::datatypes::Field, Arc<dyn Array>) {
        let value_field = Field::new("value", arrow::datatypes::DataType::UInt32, false);
        let value_arr = builder.finish();
        let value_arr = (&value_arr as &dyn Array).slice(0, value_arr.len());
        (value_field, value_arr)
    }
}

impl<'a> ArrowReadableValue<'a> for u32 {
    fn get(array: &Arc<dyn Array>, index: usize) -> u32 {
        let array = array.as_any().downcast_ref::<UInt32Array>().unwrap();
        array.value(index)
    }
    fn add_to_delta<K: ArrowWriteableKey>(
        prefix: &str,
        key: K,
        value: Self,
        builder: &mut BlockStorage,
    ) {
        u32::add(prefix, key.into(), value, builder);
    }
}
