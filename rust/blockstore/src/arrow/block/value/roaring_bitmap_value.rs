use std::sync::Arc;

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
    array::{Array, BinaryArray, BinaryBuilder},
    datatypes::Field,
    util::bit_util,
};
use roaring::RoaringBitmap;

impl ArrowWriteableValue for RoaringBitmap {
    type ReadableValue<'referred_data> = RoaringBitmap;
    type ArrowBuilder = BinaryBuilder;
    type ArrowCapacityHint = SingleColumnStorageArrowValueCapacityHint;
    type PreparedValue = Vec<u8>;

    fn offset_size(item_count: usize) -> usize {
        bit_util::round_upto_multiple_of_64((item_count + 1) * 4)
    }

    fn validity_size(_item_count: usize) -> usize {
        0 // We don't support None values for RoaringBitmap
    }

    fn add(prefix: &str, key: KeyWrapper, value: Self, delta: &BlockStorage) {
        match &delta {
            BlockStorage::RoaringBitmap(builder) => {
                builder.add(prefix, key, value);
            }
            _ => panic!("Invalid builder type"),
        }
    }

    fn delete(prefix: &str, key: KeyWrapper, delta: &BlockDelta) {
        match &delta.builder {
            BlockStorage::RoaringBitmap(builder) => {
                builder.delete(prefix, key);
            }
            _ => panic!("Invalid builder type"),
        }
    }

    fn get_delta_builder(mutation_ordering_hint: BuilderMutationOrderHint) -> BlockStorage {
        BlockStorage::RoaringBitmap(SingleColumnStorage::new(mutation_ordering_hint))
    }

    fn get_arrow_builder(capacity_hint: Self::ArrowCapacityHint) -> Self::ArrowBuilder {
        BinaryBuilder::with_capacity(capacity_hint.item_count, capacity_hint.byte_size)
    }

    fn prepare(value: Self) -> Self::PreparedValue {
        let mut serialized = Vec::with_capacity(value.serialized_size());
        if value.serialize_into(&mut serialized).is_err() {
            // todo: proper error handling
            panic!("Failed to serialize RoaringBitmap");
        }

        serialized
    }

    fn append(value: Self::PreparedValue, builder: &mut Self::ArrowBuilder) {
        builder.append_value(value);
    }

    fn finish(mut builder: Self::ArrowBuilder) -> (Field, Arc<dyn Array>) {
        let value_field = Field::new("value", arrow::datatypes::DataType::Binary, true);
        let value_arr = builder.finish();
        let value_arr = (&value_arr as &dyn Array).slice(0, value_arr.len());
        (value_field, value_arr)
    }
}

impl ArrowReadableValue<'_> for RoaringBitmap {
    fn get(array: &std::sync::Arc<dyn Array>, index: usize) -> Self {
        let arr = array.as_any().downcast_ref::<BinaryArray>().unwrap();
        let bytes = arr.value(index);
        // TODO: proper error handling
        RoaringBitmap::deserialize_from(bytes).unwrap()
    }

    fn add_to_delta<K: ArrowWriteableKey>(
        prefix: &str,
        key: K,
        value: Self,
        delta: &mut BlockStorage,
    ) {
        RoaringBitmap::add(prefix, key.into(), value, delta);
    }
}
