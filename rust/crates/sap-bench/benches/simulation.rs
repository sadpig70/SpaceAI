//! SimulationEngine 벤치마크

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::Rng;
use sap_core::types::{Position, Velocity};
use sap_physvisor::SimulationEngine;

fn random_position(rng: &mut impl Rng, max: f32) -> Position {
    Position::new(rng.gen_range(0.0..max), rng.gen_range(0.0..max), 0.0)
}

fn bench_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("SimulationEngine::step");

    for robot_count in [10, 100, 500].iter() {
        group.throughput(Throughput::Elements(*robot_count as u64));
        group.bench_with_input(
            BenchmarkId::new("robots", robot_count),
            robot_count,
            |b, &count| {
                let mut engine = SimulationEngine::with_default_config();
                let mut rng = rand::thread_rng();

                for i in 0..count {
                    engine.register_robot(i as u64);
                    engine.update_robot(i as u64, random_position(&mut rng, 100.0), Velocity::ZERO);
                }

                b.iter(|| engine.step());
            },
        );
    }
    group.finish();
}

fn bench_collision_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("SimulationEngine::collision");

    // 밀집된 로봇 배치로 충돌 감지 테스트
    group.bench_function("dense_10_robots", |b| {
        let mut engine = SimulationEngine::with_default_config();

        // 좁은 영역에 로봇 배치
        for i in 0..10 {
            engine.register_robot(i);
            engine.update_robot(
                i,
                Position::new((i % 3) as f32 * 0.3, (i / 3) as f32 * 0.3, 0.0),
                Velocity::ZERO,
            );
        }

        b.iter(|| engine.step());
    });

    group.bench_function("sparse_100_robots", |b| {
        let mut engine = SimulationEngine::with_default_config();
        let mut rng = rand::thread_rng();

        // 넓은 영역에 로봇 배치
        for i in 0..100 {
            engine.register_robot(i);
            engine.update_robot(i, random_position(&mut rng, 1000.0), Velocity::ZERO);
        }

        b.iter(|| engine.step());
    });

    group.finish();
}

fn bench_zone_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("SimulationEngine::zone");

    group.bench_function("update_100_robots", |b| {
        let mut engine = SimulationEngine::with_default_config();
        engine.add_zone(1, 0.0, 50.0, 0.0, 50.0);
        engine.add_zone(2, 50.0, 100.0, 0.0, 50.0);

        let mut rng = rand::thread_rng();
        for i in 0..100 {
            engine.register_robot(i);
        }

        b.iter(|| {
            for i in 0..100u64 {
                engine.update_robot(i, random_position(&mut rng, 100.0), Velocity::ZERO);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_step,
    bench_collision_detection,
    bench_zone_management
);
criterion_main!(benches);
