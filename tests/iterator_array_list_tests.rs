use aura_lang::compile;
use std::fs;
use std::process::Command;

#[test]
fn test_array_list_map_filter_sort_typecheck_and_dts() {
    let source = r#"
        export fn transformList(items: List<Int>): List<Int> {
            return items
                .filter(fn(x: Int): Bool => x > 2)
                .map(fn(x: Int): Int => x * 10)
                .sort();
        }

        export fn transformArray(items: Array<Int>): Array<Int> {
            return items |> filter(fn(x: Int): Bool => x % 2 == 0) |> map(fn(x: Int): Int => x + 100) |> sort();
        }
    "#;

    let res = compile(source, &[]).expect("Compilation should succeed");
    assert!(
        res.dts_code
            .contains("transformList(items: Array<number>): Array<number>;")
    );
    assert!(
        res.dts_code
            .contains("transformArray(items: Array<number>): Array<number>;")
    );
}

#[test]
fn test_custom_iterator_duck_typing() {
    let source = r#"
        interface NumberIterator {
            next(): Option<Int>;
        }

        fn drain(it: Iterator<Int>): List<Int> {
            let mut out: List<Int> = [];
            for item in it {
                out.push(item);
            }
            return out;
        }

        export fn run(): List<Int> {
            let mut count = 0;
            let myIter = {
                next: fn(): Option<Int> => {
                    if count < 3 {
                        count = count + 1;
                        return Some(count);
                    }
                    return None;
                }
            };
            return drain(myIter);
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of duck typing Iterator should succeed");
    assert!(res.dts_code.contains("interface Iterator"));
}

#[test]
fn test_interator_alias_duck_typing() {
    let source = r#"
        fn consume(it: Interator<String>): Int {
            let mut c = 0;
            for s in it {
                c = c + 1;
            }
            return c;
        }

        export fn run(): Int {
            let words: Array<String> = ["hello", "world"];
            return consume(words);
        }
    "#;

    let res = compile(source, &[]).expect("Compilation of Interator alias should succeed");
    assert!(
        res.dts_code
            .contains("export declare type Interator<T> = Iterator<T>;")
    );
}

#[test]
fn test_e2e_array_list_map_filter_sort_node() {
    let source = r#"
        export fn testArrayMethods(): List<Int> {
            let numbers: Array<Int> = [5, 2, 8, 1, 9, 3];
            let evens = numbers.filter(fn(x: Int): Bool => x % 2 == 0);
            let doubled = evens.map(fn(x: Int): Int => x * 10);
            return doubled.sort();
        }

        export fn testPipeline(): List<Int> {
            let raw: List<Int> = [50, 10, 40, 20, 30];
            return raw
                |> filter(fn(n: Int): Bool => n >= 20)
                |> map(fn(n: Int): Int => n / 10)
                |> sort();
        }

        export fn testStandalone(): List<Int> {
            let nums = [9, 3, 7];
            let m = map(nums, fn(x: Int): Int => x + 1);
            let f = filter(m, fn(x: Int): Bool => x > 5);
            return sort(f);
        }

        export fn testNumericSort(): List<Int> {
            // Must sort numerically, not lexicographically (so 100 comes after 20)
            let unordered = [100, 20, 5, 200, 15];
            return unordered.sort();
        }

        export fn main(): Bool {
            let mRes = testArrayMethods();
            // evens of [5,2,8,1,9,3] are [2,8] -> doubled [20,80] -> sorted [20,80]
            if (mRes.length != 2 || mRes[0] != 20 || mRes[1] != 80) {
                return false;
            }

            let pRes = testPipeline();
            // raw: [50, 10, 40, 20, 30] >= 20: [50, 40, 20, 30] -> /10: [5, 4, 2, 3] -> sort: [2, 3, 4, 5]
            if (pRes.length != 4 || pRes[0] != 2 || pRes[1] != 3 || pRes[2] != 4 || pRes[3] != 5) {
                return false;
            }

            let sRes = testStandalone();
            // [9,3,7] +1: [10,4,8] >5: [10,8] sorted: [8,10]
            if (sRes.length != 2 || sRes[0] != 8 || sRes[1] != 10) {
                return false;
            }

            let nRes = testNumericSort();
            // [100, 20, 5, 200, 15] sorted: [5, 15, 20, 100, 200]
            if (nRes[0] != 5 || nRes[1] != 15 || nRes[2] != 20 || nRes[3] != 100 || nRes[4] != 200) {
                return false;
            }

            return true;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nconst ok = main();\nif (!ok) { console.error('main returned false'); process.exit(1); }\n");

    let tmp_file = "test_array_list_sort.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(
        status.success(),
        "Node execution of Array/List map/filter/sort failed"
    );
}

#[test]
fn test_e2e_iterator_operations_node() {
    let source = r#"
        export fn main(): Bool {
            // Test iter() wrapping and chaining on iterator
            let it = iter([30, 10, 50, 20, 40]);
            let res = it
                .filter(fn(x: Int): Bool => x > 15)
                .map(fn(x: Int): Int => x * 2)
                .sort()
                .toList();

            // > 15: [30, 50, 20, 40]
            // * 2: [60, 100, 40, 80]
            // sort: [40, 60, 80, 100]
            if (res.length != 4 || res[0] != 40 || res[1] != 60 || res[2] != 80 || res[3] != 100) {
                return false;
            }

            // Test custom iterator with for..in loop
            let mut i = 0;
            let counter = {
                next: fn(): Option<Int> => {
                    if i < 3 {
                        i = i + 1;
                        return Some(i * 10);
                    }
                    return None;
                }
            };

            let mut sum = 0;
            for val in counter {
                sum = sum + val;
            }

            // sum should be 10 + 20 + 30 = 60
            if (sum != 60) {
                return false;
            }

            return true;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nconst ok = main();\nif (!ok) { console.error('main returned false'); process.exit(1); }\n");

    let tmp_file = "test_iterator_ops.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(
        status.success(),
        "Node execution of Iterator operations failed"
    );
}

#[test]
fn test_custom_comparator_and_immutability() {
    let source = r#"
        export fn main(): Bool {
            let original: Array<Int> = [3, 1, 4, 1, 5, 9, 2, 6];
            // Sort descending with custom comparator
            let desc = original.sort(fn(a: Int, b: Int): Int => b - a);

            // Verify immutability of the original array
            if (original[0] != 3 || original[1] != 1) {
                return false;
            }

            // Verify descending order
            if (desc[0] != 9 || desc[1] != 6 || desc[2] != 5 || desc[3] != 4 || desc[4] != 3 || desc[5] != 2 || desc[6] != 1 || desc[7] != 1) {
                return false;
            }

            return true;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nconst ok = main();\nif (!ok) { console.error('desc sort failed'); process.exit(1); }\n");

    let tmp_file = "test_custom_cmp.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(
        status.success(),
        "Node execution of custom comparator / immutability failed"
    );
}

#[test]
fn test_custom_iterator_with_map_filter_sort_pipeline() {
    let source = r#"
        export fn main(): Bool {
            let mut current = 1;
            let rangeIter = {
                next: fn(): Option<Int> => {
                    if current <= 5 {
                        let val = current;
                        current = current + 1;
                        return Some(val);
                    }
                    return None;
                }
            };

            // Process custom iterator through pipeline
            let result = rangeIter
                |> filter(fn(x: Int): Bool => x % 2 == 1)
                |> map(fn(x: Int): Int => x * 10)
                |> sort(fn(a: Int, b: Int): Int => b - a)
                |> toList();

            // range 1..5 odd numbers are 1, 3, 5 -> * 10: 10, 30, 50 -> desc sort: 50, 30, 10
            if (result.length != 3 || result[0] != 50 || result[1] != 30 || result[2] != 10) {
                return false;
            }

            return true;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation failed");
    let mut js_runner = String::new();
    js_runner.push_str(&res.js_code);
    js_runner.push_str("\n\nconst ok = main();\nif (!ok) { console.error('custom iterator pipeline failed'); process.exit(1); }\n");

    let tmp_file = "test_custom_iter_pipeline.tmp.mjs";
    fs::write(tmp_file, js_runner).expect("Failed to write tmp runner");

    let status = Command::new("node")
        .arg(tmp_file)
        .status()
        .expect("Failed to execute node");

    let _ = fs::remove_file(tmp_file);
    assert!(
        status.success(),
        "Node execution of custom iterator pipeline failed"
    );
}
