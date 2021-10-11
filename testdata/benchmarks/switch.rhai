let arr = [42, 123.456, "hello", true, 'x', 999, 1];

for item in arr {
    switch item {
        42 => print("The Answer!"),
        123.456 => print(`Floating point... ${item}`),
        "hello" => print(`${item} world!`),
        999 => print(`A number: ${item}`),
        _ => print(`Something else: <${item}>`)
    }
}
