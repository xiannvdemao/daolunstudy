use sqlrustgo::{parse, ExecutionEngine};
use std::time::Instant;

const TEST_QUERIES: usize = 10000;

#[ignore]
#[test]
fn test_qps_delete() {
    let mut engine = ExecutionEngine::new();

    // 准备测试表和初始数据
    engine
        .execute(parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
        .unwrap();
    for i in 0..TEST_QUERIES {
        engine
            .execute(parse(&format!("INSERT INTO users VALUES ({}, 'user{}')", i, i)).unwrap())
            .unwrap();
    }

    // 执行DELETE测试
    let start = Instant::now();
    for i in 0..TEST_QUERIES {
        engine
            .execute(parse(&format!("DELETE FROM users WHERE id = {}", i)).unwrap())
            .unwrap();
    }
    let duration = start.elapsed();
    let qps = TEST_QUERIES as f64 / duration.as_secs_f64();

    println!(
        "DELETE QPS: {} queries in {:.2}s ({:.2} qps)",
        TEST_QUERIES,
        duration.as_secs_f64(),
        qps
    );
}

#[ignore]
#[test]
fn test_qps_update() {
    let mut engine = ExecutionEngine::new();

    // 准备测试表和初始数据
    engine
        .execute(parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
        .unwrap();
    for i in 0..TEST_QUERIES {
        engine
            .execute(parse(&format!("INSERT INTO users VALUES ({}, 'user{}')", i, i)).unwrap())
            .unwrap();
    }

    // 执行UPDATE测试
    let start = Instant::now();
    for i in 0..TEST_QUERIES {
        engine
            .execute(
                parse(&format!(
                    "UPDATE users SET name = 'updated{}' WHERE id = {}",
                    i, i
                ))
                .unwrap(),
            )
            .unwrap();
    }
    let duration = start.elapsed();
    let qps = TEST_QUERIES as f64 / duration.as_secs_f64();

    println!(
        "UPDATE QPS: {} queries in {:.2}s ({:.2} qps)",
        TEST_QUERIES,
        duration.as_secs_f64(),
        qps
    );
}

#[ignore]
#[test]
fn test_qps_insert() {
    let mut engine = ExecutionEngine::new();

    // 准备测试表
    engine
        .execute(parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
        .unwrap();

    // 执行INSERT测试
    let start = Instant::now();
    for i in 0..TEST_QUERIES {
        engine
            .execute(parse(&format!("INSERT INTO users VALUES ({}, 'user{}')", i, i)).unwrap())
            .unwrap();
    }
    let duration = start.elapsed();
    let qps = TEST_QUERIES as f64 / duration.as_secs_f64();

    println!(
        "INSERT QPS: {} queries in {:.2}s ({:.2} qps)",
        TEST_QUERIES,
        duration.as_secs_f64(),
        qps
    );
}

#[ignore]
#[test]
fn test_qps_simple_select() {
    let mut engine = ExecutionEngine::new();

    // 准备测试表和数据
    engine
        .execute(parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'test')").unwrap())
        .unwrap();

    // 执行SELECT测试
    let start = Instant::now();
    for _ in 0..TEST_QUERIES {
        engine
            .execute(parse("SELECT * FROM users WHERE id = 1").unwrap())
            .unwrap();
    }
    let duration = start.elapsed();
    let qps = TEST_QUERIES as f64 / duration.as_secs_f64();

    println!(
        "SELECT QPS: {} queries in {:.2}s ({:.2} qps)",
        TEST_QUERIES,
        duration.as_secs_f64(),
        qps
    );
}
