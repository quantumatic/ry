mod r#macro;

test!(r#return: "fun test(): int32 { return a()?.b.unwrap_or(0); }");
test!(let_and_defer: "fun test() {
    let file = File.open(\"hello.txt\");
    defer file.close();
}");
test!(let1: "fun test(): int32 {
    let a = 1;
    let b = 2;
    a + b
}");
test!(let2: "fun test(): int32 {
    let Some(a) = Some(2);
}");
// grouped pattern
test!(let3: "fun test(): int32 {
    let (Some(a)) = Some(2);
}");
test!(let4: "fun test(): int32 {
    let #(Some(a), None) = #(Some(2), None);
}");
// or
test!(let5: "fun test(): int32 {
    let A(a) | B(a) = A(2);
}");
test!(let6: "fun test(): int32 {
    let a: Option[int32] = Some(2);
}");