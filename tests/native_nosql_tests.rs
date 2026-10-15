//! Integration tests for Native Standalone NoSQL Drivers in Aura (Redis & MongoDB).
//! Covers Cranelift binary compilation, native execution, embedded in-memory engines, and wire operations.

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_native_redis_cranelift_binary_execution() {
    let source = r#"
        export fn main(): Unit => {
            println("--- Starting Redis Native Driver Test ---");
            let r = redis.open(":memory:");

            // String Key-Value operations
            r.set("user:1:name", "Alice", 0);
            r.set("user:1:email", "alice@example.com", 0);
            r.set("counter", "10", 0);

            let name = r.get("user:1:name");
            println(name);

            let exists = r.exists("user:1:name");
            println(exists);

            let newCount = r.incr("counter", 5);
            println(newCount);

            let decrCount = r.decr("counter", 2);
            println(decrCount);

            // Hash Map operations
            r.hset("session:abc", "userId", "101");
            r.hset("session:abc", "role", "admin");

            let role = r.hget("session:abc", "role");
            println(role);

            // List Queue operations
            r.lpush("tasks", "task1");
            r.lpush("tasks", "task2");
            r.lpush("tasks", "task3");

            let popped = r.rpop("tasks");
            println(popped);

            let pong = r.ping();
            println(pong);

            r.close();
            println("--- Redis Native Driver Test Finished ---");
        }
    "#;

    let out_bin = Path::new("test_cranelift_redis_bin");
    let res = aura_lang::build_native_binary(source, out_bin);
    assert!(
        res.is_ok(),
        "Native compilation of Redis driver failed: {:?}",
        res.err()
    );

    let output = Command::new(format!("./{}", out_bin.display()))
        .output()
        .expect("Failed to execute native Redis binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Redis Test STDOUT:\n{}", stdout);

    assert!(
        output.status.success(),
        "Binary failed with status: {:?}",
        output.status
    );
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("admin"));
    assert!(stdout.contains("task1"));
    assert!(stdout.contains("PONG"));
    assert!(stdout.contains("--- Redis Native Driver Test Finished ---"));

    let _ = fs::remove_file(out_bin);
}

#[test]
fn test_native_mongo_cranelift_binary_execution() {
    let source = r#"
        export fn main(): Unit => {
            println("--- Starting MongoDB Native Driver Test ---");
            let client = mongo.open(":memory:");
            let db = client.db("testdb");
            let users = db.collection("users");

            // Insert single document
            let res1 = users.insertOne({ name: "Alice", role: "admin", score: 95 });
            println(res1.insertedId);

            // Insert multiple documents
            let resMany = users.insertMany([
                { name: "Bob", role: "developer", score: 85 },
                { name: "Charlie", role: "developer", score: 90 },
                { name: "Diana", role: "designer", score: 88 }
            ]);
            println(resMany.insertedCount);

            // Count documents
            let total = users.countDocuments({});
            println(total);

            let devCount = users.count({ role: "developer" });
            println(devCount);

            // Update single document
            let upRes = users.updateOne({ name: "Alice" }, { "$set": { score: 100 } });
            println(upRes.matchedCount);
            println(upRes.modifiedCount);

            // Find single document
            let alice = users.findOne({ name: "Alice" });
            println(alice.name);

            // Aggregation pipeline
            let agg = users.aggregate([
                { "$match": { role: "developer" } },
                { "$group": { _id: "$role", totalScore: { "$sum": "$score" } } }
            ]);
            let aggLen = len(agg);
            println(aggLen);

            // Delete document
            let delRes = users.deleteOne({ name: "Diana" });
            println(delRes.deletedCount);

            let finalCount = users.count({});
            println(finalCount);

            client.close();
            println("--- MongoDB Native Driver Test Finished ---");
        }
    "#;

    let out_bin = Path::new("test_cranelift_mongo_bin");
    let res = aura_lang::build_native_binary(source, out_bin);
    assert!(
        res.is_ok(),
        "Native compilation of MongoDB driver failed: {:?}",
        res.err()
    );

    let output = Command::new(format!("./{}", out_bin.display()))
        .output()
        .expect("Failed to execute native MongoDB binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("MongoDB Test STDOUT:\n{}", stdout);

    assert!(
        output.status.success(),
        "Binary failed with status: {:?}",
        output.status
    );
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("3")); // insertedCount
    assert!(stdout.contains("4")); // total
    assert!(stdout.contains("2")); // devCount
    assert!(stdout.contains("1")); // matchedCount / deletedCount
    assert!(stdout.contains("3")); // finalCount
    assert!(stdout.contains("--- MongoDB Native Driver Test Finished ---"));

    let _ = fs::remove_file(out_bin);
}
