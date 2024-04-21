use std::any::TypeId;

use acro_ecs::{
    entity::EntityId,
    storage::{DowncastedStorage, Storage},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn storage_retrieval(storage: &DowncastedStorage<'_, String>) {
    assert_eq!(storage.get(EntityId(0)).unwrap(), "hello");
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut storage = Storage::new(TypeId::of::<String>());
    storage.insert(EntityId(0), "hello".to_string());
    storage.insert(EntityId(1), "bye".to_string());

    let downcasted = storage.downcast().expect("downcast failed");

    c.bench_function("fib 20", |b| {
        b.iter(|| storage_retrieval(black_box(&downcasted)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
