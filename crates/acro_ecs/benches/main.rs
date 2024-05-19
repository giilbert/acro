use acro_ecs::world::World;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("insert 1000", |cx| {
        let mut world = World::new();
        world.init_component::<u32>();

        cx.iter(|| {
            for i in 0..1000 {
                let entity = black_box(world.spawn());
                black_box(world.insert(entity, i as u32));
            }
        });
    });

    c.bench_function("query 1000", |cx| {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<String>();

        for i in 0..1000 {
            let entity = world.spawn();
            world.insert(entity, i as u32);
            world.insert(entity, i.to_string());
        }

        cx.iter(|| {
            let query = world.query::<(&u32, &String), ()>();
            for value in query.exclusive(&mut world) {
                black_box(value);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
