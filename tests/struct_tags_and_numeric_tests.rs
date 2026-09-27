use aura_lang::compile;

#[test]
fn test_fixed_size_numeric_types() {
    let source = r#"
        export fn numericCalculations(
            a: Int8,
            b: Int16,
            c: Int32,
            d: Int64,
            u8: Uint8,
            u16: Uint16,
            u32: Uint32,
            u64: Uint64,
            bt: Byte,
            rn: Rune,
            f32: Float32,
            f64: Float64
        ): Float64 {
            let total = (a + b + c + d + u8 + u16 + u32 + u64 + bt + rn) + (f32 + f64);
            return total;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation with fixed-size numeric types failed");
    assert!(res.dts_code.contains("a: number"));
    assert!(res.dts_code.contains("u8: number"));
}

#[test]
fn test_struct_tags_definition_and_access() {
    let source = r#"
        export type User = {
            id: Int `json:"id" validate:"required"`,
            name: String `json:"name"`,
            email: String `json:"email" validate:"email"`
        };

        export fn validateUser(u: User): Bool {
            return true;
        }
    "#;

    let res = compile(source, &[]).expect("Compilation with struct tags failed");
    assert!(res.dts_code.contains("id: number"));
    assert!(res.dts_code.contains("name: string"));
}
