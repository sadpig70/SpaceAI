//! SAP Warehouse Demo - Simplified Standalone Version
#![allow(clippy::manual_is_multiple_of)]
//!
//! 5 robots, 10x10m warehouse, 20 pickup tasks
//! Distance-based VTS allocation, Cross-Zone detection, Collision detection

use sap_core::types::{Position, Velocity};
use std::time::{Duration, Instant};

// === Configuration ===
const NUM_ROBOTS: usize = 5;
const WAREHOUSE_SIZE: (f32, f32) = (10.0, 10.0);
const NUM_TASKS: usize = 20;
const SIMULATION_DURATION_SECS: u64 = 60;
const TICK_INTERVAL_MS: u64 = 100;

// === Robot ===
#[derive(Debug, Clone)]
struct Robot {
    id: u64,
    position: Position,
    velocity: Velocity,
    assigned_task: Option<usize>,
}

impl Robot {
    fn new(id: u64, x: f32, y: f32) -> Self {
        Self {
            id,
            position: Position::new(x, y, 0.0),
            velocity: Velocity::new(0.0, 0.0, 0.0),
            assigned_task: None,
        }
    }

    fn update(&mut self, dt: f32) {
        self.position.x += self.velocity.vx * dt;
        self.position.y += self.velocity.vy * dt;
        self.position.x = self.position.x.clamp(0.0, WAREHOUSE_SIZE.0);
        self.position.y = self.position.y.clamp(0.0, WAREHOUSE_SIZE.1);
    }

    fn move_to(&mut self, target: Position) {
        let dx = target.x - self.position.x;
        let dy = target.y - self.position.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > 0.1 {
            let speed = 1.0;
            self.velocity.vx = (dx / dist) * speed;
            self.velocity.vy = (dy / dist) * speed;
        } else {
            self.velocity = Velocity::new(0.0, 0.0, 0.0);
        }
    }

    /// 충돌 회피: 다른 로봇과 너무 가까우면 속도 감속
    fn avoid_collision(&mut self, other_robots: &[Robot]) {
        const SAFETY_DISTANCE: f32 = 1.5; // 안전 거리 증가
        const SLOW_DISTANCE: f32 = 2.5; // 감속 시작 거리

        let mut closest_dist = f32::MAX;

        for other in other_robots {
            if other.id == self.id {
                continue;
            }

            let dx = other.position.x - self.position.x;
            let dy = other.position.y - self.position.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < closest_dist {
                closest_dist = dist;
            }
        }

        // 안전 거리 기반 속도 조정
        if closest_dist < SAFETY_DISTANCE {
            // 매우 가까움: 정지
            self.velocity.vx *= 0.1;
            self.velocity.vy *= 0.1;
        } else if closest_dist < SLOW_DISTANCE {
            // 가까움: 감속 (거리 비례)
            let slow_factor = (closest_dist - SAFETY_DISTANCE) / (SLOW_DISTANCE - SAFETY_DISTANCE);
            self.velocity.vx *= slow_factor.max(0.3);
            self.velocity.vy *= slow_factor.max(0.3);
        }
    }
}

// === Task ===
#[derive(Debug, Clone)]
struct Task {
    id: usize,
    pickup: Position,
    assigned_robot: Option<u64>,
    completed: bool,
}

impl Task {
    fn new(id: usize, px: f32, py: f32) -> Self {
        Self {
            id,
            pickup: Position::new(px, py, 0.0),
            assigned_robot: None,
            completed: false,
        }
    }
}

// === Metrics ===
#[derive(Debug, Default)]
struct Metrics {
    tasks_completed: usize,
    allocations: usize,
    collisions: usize,
    handoffs: usize,
}

impl Metrics {
    fn throughput(&self, elapsed: f32) -> f32 {
        if elapsed > 0.0 {
            self.tasks_completed as f32 / elapsed
        } else {
            0.0
        }
    }

    fn collision_rate(&self) -> f32 {
        if self.allocations > 0 {
            self.collisions as f32 / self.allocations as f32 * 100.0
        } else {
            0.0
        }
    }
}

// === Simulator ===
struct Simulator {
    robots: Vec<Robot>,
    tasks: Vec<Task>,
    metrics: Metrics,
    tick: u64,
}

impl Simulator {
    fn new() -> Self {
        let robots = vec![
            Robot::new(1, 0.0, 0.0),
            Robot::new(2, 10.0, 0.0),
            Robot::new(3, 0.0, 10.0),
            Robot::new(4, 10.0, 10.0),
            Robot::new(5, 5.0, 5.0),
        ];

        let mut tasks = Vec::new();
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for id in 0..NUM_TASKS {
            let px = rng.gen_range(0.0..WAREHOUSE_SIZE.0);
            let py = rng.gen_range(0.0..WAREHOUSE_SIZE.1);
            tasks.push(Task::new(id, px, py));
        }

        Self {
            robots,
            tasks,
            metrics: Metrics::default(),
            tick: 0,
        }
    }

    fn allocate_tasks(&mut self) {
        for task in &mut self.tasks {
            if task.assigned_robot.is_none() && !task.completed {
                let mut best_robot = None;
                let mut min_dist = f32::MAX;

                for robot in &self.robots {
                    if robot.assigned_task.is_none() {
                        let dx = task.pickup.x - robot.position.x;
                        let dy = task.pickup.y - robot.position.y;
                        let dist = (dx * dx + dy * dy).sqrt();

                        if dist < min_dist {
                            min_dist = dist;
                            best_robot = Some(robot.id);
                        }
                    }
                }

                if let Some(rid) = best_robot {
                    task.assigned_robot = Some(rid);
                    if let Some(r) = self.robots.iter_mut().find(|r| r.id == rid) {
                        r.assigned_task = Some(task.id);
                    }
                    self.metrics.allocations += 1;
                    println!(
                        "[{:05}] VTS: Robot #{} → Task #{} ({:.1}m)",
                        self.tick, rid, task.id, min_dist
                    );
                }
            }
        }
    }

    fn detect_collisions(&mut self) {
        const COLLISION_THRESHOLD: f32 = 1.0; // 충돌 판정 거리 증가

        for i in 0..self.robots.len() {
            for j in (i + 1)..self.robots.len() {
                let r1 = &self.robots[i];
                let r2 = &self.robots[j];
                let dx = r1.position.x - r2.position.x;
                let dy = r1.position.y - r2.position.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < COLLISION_THRESHOLD {
                    self.metrics.collisions += 1;
                    println!(
                        "[{:05}] ⚠️  Collision: R#{} & R#{} ({:.2}m)",
                        self.tick, r1.id, r2.id, dist
                    );
                }
            }
        }
    }

    fn check_handoffs(&mut self) {
        for robot in &self.robots {
            if (robot.position.x - 5.0).abs() < 0.1 {
                self.metrics.handoffs += 1;
                println!("[{:05}] 🔄 Handoff: R#{} at boundary", self.tick, robot.id);
            }
        }
    }

    fn update_robots(&mut self, dt: f32) {
        // 1단계: 목표 방향 설정
        for i in 0..self.robots.len() {
            if let Some(tid) = self.robots[i].assigned_task {
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == tid) {
                    if !task.completed {
                        self.robots[i].move_to(task.pickup);
                    }
                }
            }
        }

        // 2단계: 충돌 회피 (모든 로봇 고려)
        for i in 0..self.robots.len() {
            let others: Vec<Robot> = self
                .robots
                .iter()
                .filter(|r| r.id != self.robots[i].id)
                .cloned()
                .collect();
            self.robots[i].avoid_collision(&others);
        }

        // 3단계: 위치 업데이트 및 태스크 완료 확인
        for robot in &mut self.robots {
            robot.update(dt);

            if let Some(tid) = robot.assigned_task {
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == tid) {
                    if !task.completed {
                        let dx = task.pickup.x - robot.position.x;
                        let dy = task.pickup.y - robot.position.y;
                        let dist = (dx * dx + dy * dy).sqrt();

                        if dist < 0.2 {
                            task.completed = true;
                            robot.assigned_task = None;
                            self.metrics.tasks_completed += 1;
                            println!(
                                "[{:05}] ✅ Task #{} done by R#{}",
                                self.tick, task.id, robot.id
                            );
                        }
                    }
                }
            }
        }
    }

    fn render(&self) {
        const GRID: usize = 20;
        let mut grid = vec![vec![' '; GRID]; GRID];

        for (i, r) in self.robots.iter().enumerate() {
            let x = ((r.position.x / WAREHOUSE_SIZE.0) * GRID as f32) as usize;
            let y = ((r.position.y / WAREHOUSE_SIZE.1) * GRID as f32) as usize;
            if x < GRID && y < GRID {
                grid[GRID - 1 - y][x] = char::from_digit((i + 1) as u32, 10).unwrap();
            }
        }

        let bx = ((5.0 / WAREHOUSE_SIZE.0) * GRID as f32) as usize;
        for row in &mut grid {
            if row[bx] == ' ' {
                row[bx] = '|';
            }
        }

        println!("\n╔{}╗", "═".repeat(GRID));
        for row in &grid {
            print!("║");
            for &c in row {
                print!("{}", c);
            }
            println!("║");
        }
        println!("╚{}╝", "═".repeat(GRID));
        println!("  Zone A    |    Zone B");
    }

    fn run(&mut self) {
        println!("=== SAP Warehouse Demo ===");
        println!(
            "Robots: {}, Tasks: {}, Duration: {}s\n",
            NUM_ROBOTS, NUM_TASKS, SIMULATION_DURATION_SECS
        );

        let start = Instant::now();
        let tick_dur = Duration::from_millis(TICK_INTERVAL_MS);
        let dt = TICK_INTERVAL_MS as f32 / 1000.0;

        while start.elapsed().as_secs() < SIMULATION_DURATION_SECS {
            self.tick += 1;

            if self.tick % 10 == 0 {
                self.allocate_tasks();
            }

            self.update_robots(dt);
            self.check_handoffs();

            if self.tick % 10 == 0 {
                self.detect_collisions();
            }

            if self.tick % 50 == 0 {
                self.render();
            }

            std::thread::sleep(tick_dur);

            if self.tasks.iter().all(|t| t.completed) {
                println!("\n🎉 All tasks completed!");
                break;
            }
        }

        self.print_metrics(start.elapsed().as_secs_f32());
    }

    fn print_metrics(&self, elapsed: f32) {
        println!("\n{}", "=".repeat(50));
        println!("📊 Final Metrics");
        println!("{}", "=".repeat(50));
        println!(
            "Tasks Completed:  {}/{}",
            self.metrics.tasks_completed, NUM_TASKS
        );
        println!(
            "Throughput:       {:.3} tasks/sec",
            self.metrics.throughput(elapsed)
        );
        println!("Allocations:      {}", self.metrics.allocations);
        println!("Handoffs:         {}", self.metrics.handoffs);
        println!("Collisions:       {}", self.metrics.collisions);
        println!("Collision Rate:   {:.1}%", self.metrics.collision_rate());
        println!("Elapsed Time:     {:.1}s", elapsed);
        println!("{}", "=".repeat(50));
    }
}

fn main() {
    let mut sim = Simulator::new();
    sim.run();
}
