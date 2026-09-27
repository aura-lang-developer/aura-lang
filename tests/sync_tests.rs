use aura_lang::compile;
use std::fs;
use std::process::Command;

#[test]
fn test_sync_primitives_typecheck_and_codegen() {
    let source = r#"
        export async fn runSyncDemo() {
            let mu = Mutex.new();
            await mu.lock();
            let locked = mu.tryLock();
            mu.unlock();

            let rw = RWMutex.new();
            await rw.rLock();
            rw.rUnlock();

            let once = Once.new();
            once.do(() => {
                println("Executed exactly once");
            });

            let pool = Pool.new(() => {
                return { count: 0 };
            });
            let item = pool.get();
            pool.put(item);
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of Sync module failed");
    assert!(res.js_code.contains("Mutex.new()"));
    assert!(res.js_code.contains("RWMutex.new()"));
    assert!(res.js_code.contains("Once.new()"));
    assert!(res.js_code.contains("Pool.new("));
}

#[test]
fn test_e2e_sync_primitives_with_node() {
    let source = r#"
        export async fn testMutex(): Task<Int, String> {
            let mu = Mutex.new();
            let mut counter = 0;
            let wg = WaitGroup.new();
            wg.add(5);

            for (i in [0, 1, 2, 3, 4]) {
                spawn(async () => {
                    await mu.lock();
                    let current = counter;
                    await sleep(5);
                    counter = current + 1;
                    mu.unlock();
                    wg.done();
                });
            }

            await wg.wait();
            return counter;
        }

        export async fn testOnce(): Task<Int, String> {
            let once = Once.new();
            let mut count = 0;
            for (i in [0, 1, 2, 3]) {
                once.do(() => {
                    count = count + 1;
                });
            }
            return count;
        }

        export async fn main() {
            let c = await testMutex();
            assert(c == 5, "Mutex failed to synchronize counter");

            let onceCount = await testOnce();
            assert(onceCount == 1, "Once executed more than once");
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nawait main();\n");

    let tmp_file = "test_sync_exec.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(status.success(), "Node execution of Sync primitives failed");
}
