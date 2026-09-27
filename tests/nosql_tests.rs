use aura_lang::compile;

#[test]
fn test_mongodb_typecheck_and_codegen_basic() {
    let source = r#"
        let config: MongoConfig = {
            uri: "mongodb://localhost:27017/my_nosql_db",
            host: "localhost",
            port: 27017,
            user: "",
            password: "",
            database: "my_nosql_db"
        }

        let client: MongoClient = mongodb.createClient(config)
        let db: MongoDatabase = client.db()
        let users: MongoCollection = db.collection("users")

        async fn runMongoOperations(): Task<(), String> {
            let insertRes = await users.insertOne({ name: "Alice", role: "admin", score: 95 })
            match insertRes {
                Ok(info) => println(`Inserted doc id: ${info.insertedId}`),
                Err(err) => println(`Insert error: ${err}`)
            }

            let foundDoc = await users.findOne({ name: "Alice" })
            match foundDoc {
                Ok(opt) => {
                    match opt {
                        Some(doc) => println(`Found doc: ${doc.name}`),
                        None => println("No doc found")
                    }
                },
                Err(err) => println(`Find error: ${err}`)
            }

            let updateRes = await users.updateOne({ name: "Alice" }, { "$set": { score: 100 } })
            match updateRes {
                Ok(u) => println(`Matched: ${u.matchedCount}, Modified: ${u.modifiedCount}`),
                Err(err) => println(`Update error: ${err}`)
            }

            let countRes = await users.countDocuments({ role: "admin" })
            match countRes {
                Ok(c) => println(`Total admins: ${c}`),
                Err(err) => println(`Count error: ${err}`)
            }
        }
    "#;

    let result =
        compile(source, &[]).expect("Compilation of MongoDB basic operations should succeed");
    assert!(result.js_code.contains("mongodb.createClient"));
    assert!(result.js_code.contains("client.db"));
    assert!(result.js_code.contains("db.collection"));
    assert!(result.js_code.contains("users.insertOne"));
    assert!(result.js_code.contains("users.findOne"));
    assert!(result.js_code.contains("users.updateOne"));
    assert!(result.js_code.contains("users.countDocuments"));
    assert!(result.dts_code.contains("export interface MongoClient"));
    assert!(result.dts_code.contains("export interface MongoDatabase"));
    assert!(result.dts_code.contains("export interface MongoCollection"));
    assert!(
        result
            .dts_code
            .contains("export interface MongoInsertResult")
    );
    assert!(
        result
            .dts_code
            .contains("export interface MongoUpdateResult")
    );
    assert!(result.dts_code.contains("export declare const mongodb:"));
    assert!(
        result
            .dts_code
            .contains("export declare const mongo: typeof mongodb;")
    );
}

#[test]
fn test_mongodb_aggregate_and_batch_operations() {
    let source = r#"
        let client = mongo.open("mongodb://localhost:27017/analytics")
        let col = client.collection("events")

        async fn runAnalytics(): Task<(), String> {
            let manyRes = await col.insertMany([
                { type: "click", tag: "btn_signup" },
                { type: "view", tag: "landing" }
            ])
            match manyRes {
                Ok(info) => println(`Inserted count: ${info.insertedCount}`),
                Err(e) => println(`Insert many error: ${e}`)
            }

            let aggRes = await col.aggregate([
                { "$match": { type: "click" } },
                { "$group": { _id: "$tag", total: { "$sum": 1 } } }
            ])
            match aggRes {
                Ok(items) => println(`Aggregated groups: ${items.length}`),
                Err(e) => println(`Aggregation error: ${e}`)
            }

            let delRes = await col.deleteMany({ type: "test" })
            match delRes {
                Ok(delInfo) => println(`Deleted: ${delInfo.deletedCount}`),
                Err(e) => println(`Delete error: ${e}`)
            }
        }
    "#;

    let result = compile(source, &[])
        .expect("Compilation of MongoDB aggregation and batch operations should succeed");
    assert!(result.js_code.contains("mongo.open"));
    assert!(result.js_code.contains("client.collection"));
    assert!(result.js_code.contains("col.insertMany"));
    assert!(result.js_code.contains("col.aggregate"));
    assert!(result.js_code.contains("col.deleteMany"));
    assert!(
        result
            .dts_code
            .contains("export interface MongoInsertManyResult")
    );
    assert!(
        result
            .dts_code
            .contains("export interface MongoDeleteResult")
    );
}

#[test]
fn test_redis_typecheck_and_codegen_basic() {
    let source = r#"
        let config: RedisConfig = {
            host: "127.0.0.1",
            port: 6379,
            password: "",
            db: 0,
            uri: "redis://127.0.0.1:6379"
        }

        let cache: RedisClient = redis.createClient(config)

        async fn handleCache(): Task<(), String> {
            let setOk = await cache.set("user:session:101", "active_token_xyz", 3600)
            match setOk {
                Ok(ok) => println(`Cached: ${ok}`),
                Err(err) => println(`Cache set error: ${err}`)
            }

            let getRes = await cache.get("user:session:101")
            match getRes {
                Ok(opt) => {
                    match opt {
                        Some(token) => println(`User session token: ${token}`),
                        None => println("Session expired")
                    }
                },
                Err(err) => println(`Cache get error: ${err}`)
            }

            let incRes = await cache.incr("analytics:pageviews")
            match incRes {
                Ok(count) => println(`Pageviews count: ${count}`),
                Err(err) => println(`Incr error: ${err}`)
            }

            let hsetRes = await cache.hset("user:profile:101", "theme", "dark")
            let hgetRes = await cache.hget("user:profile:101", "theme")
            let pingRes = await cache.ping()
        }
    "#;

    let result =
        compile(source, &[]).expect("Compilation of Redis basic operations should succeed");
    assert!(result.js_code.contains("redis.createClient"));
    assert!(result.js_code.contains("cache.set"));
    assert!(result.js_code.contains("cache.get"));
    assert!(result.js_code.contains("cache.incr"));
    assert!(result.js_code.contains("cache.hset"));
    assert!(result.js_code.contains("cache.hget"));
    assert!(result.js_code.contains("cache.ping"));
    assert!(result.dts_code.contains("export interface RedisClient"));
    assert!(result.dts_code.contains("export declare const redis:"));
}
