//! EdgeRuntime 벤치마크

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sap_core::types::{Acceleration, Position, Velocity};
use sap_edge::runtime::CommandResult;
use sap_edge::EdgeRuntime;
use sap_physics::command::MotionCommand;

fn create_command(robot_id: u64) -> MotionCommand {
    MotionCommand {
        robot_id,
        current_position: Position::ORIGIN,
        target_velocity: Velocity::new(1.0, 0.0, 0.0),
        target_acceleration: Acceleration::new(0.5, 0.0, 0.0),
        ticket_id: 1,
    }
}

fn bench_process_command(c: &mut Criterion) {
    let mut group = c.benchmark_group("EdgeRuntime::process_command");

    for robot_count in [1, 10, 100].iter() {
        group.throughput(Throughput::Elements(*robot_count as u64));
        group.bench_with_input(
            BenchmarkId::new("robots", robot_count),
            robot_count,
            |b, &count| {
                let mut runtime = EdgeRuntime::new(1);
                let commands: Vec<_> = (0..count).map(|i| create_command(i as u64)).collect();

                b.iter(|| {
                    for (i, cmd) in commands.iter().enumerate() {
                        let _ = runtime.process_command(cmd, i as u64 * 20_000_000);
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("EdgeRuntime::tick");

    group.bench_function("single_tick", |b| {
        let mut runtime = EdgeRuntime::new(1);
        let mut tick = 0u64;

        b.iter(|| {
            tick += 1;
            runtime.tick(tick * 20_000_000);
        });
    });

    group.finish();
}

fn bench_auction_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("EdgeRuntime::auction");

    for bid_count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*bid_count as u64));
        group.bench_with_input(
            BenchmarkId::new("bids", bid_count),
            bid_count,
            |b, &count| {
                b.iter(|| {
                    let mut runtime = EdgeRuntime::new(1);

                    for i in 0..count {
                        let _ = runtime.submit_bid(
                            i as u64,
                            100,
                            (i * 10 + 100) as u64,
                            i as u64 * 1_000_000,
                        );
                    }

                    runtime.settle_auction(100, 1_000_000_000)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_process_command,
    bench_tick,
    bench_auction_flow
);
criterion_main!(benches);
