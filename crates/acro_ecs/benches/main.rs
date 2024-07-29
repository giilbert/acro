use acro_ecs::world::World;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("insert 1000", |cx| {
        let mut world = World::new();
        world.init_component::<u32>();

        cx.iter(|| {
            for i in 0..1000 {
                let entity = black_box(world.spawn_empty());
                black_box(world.insert(entity, i as u32));
            }
        });
    });

    c.bench_function("query 1_000_000", |cx| {
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<String>();

        for i in 0..1_000_000 {
            let entity = world.spawn_empty();
            world.insert(entity, i as u32);
            world.insert(entity, i.to_string());
        }

        cx.iter(|| {
            let query = world.query::<(&u32, &String), ()>();
            for value in query.over(&world) {
                black_box(value);
            }
        });
    });

    c.bench_function("prepare queries 1_000", |cx| {
        // create a bunch of archetypes
        let mut world = World::new();
        world.init_component::<u32>();
        world.init_component::<i8>();
        world.init_component::<bool>();
        world.init_component::<String>();

        for i in 0..120 {
            let entity = world.spawn_empty();
            if i % 2 == 0 {
                world.insert(entity, i as u32);
            }
            if i % 3 == 0 {
                world.insert(entity, i as i8);
            }
            if i % 4 == 0 {
                world.insert(entity, i % 2 == 0);
            }
            if i % 5 == 0 {
                world.insert(entity, i.to_string());
            }
        }

        cx.iter(|| {
            for _ in 0..1_000 {
                let _query1 = world.query::<(&u32, &i8), ()>();
                let _query2 = world.query::<(&u32, &bool), ()>();
                let _query3 = world.query::<(&i8, &bool), ()>();
                let _query4 = world.query::<(&u32, &bool, &String), ()>();
                let _query5 = world.query::<(&i8, &bool, &String), ()>();
                let _query6 = world.query::<(&bool, &u32), ()>();
                black_box((_query1, _query2, _query3, _query4, _query5, _query6));
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
