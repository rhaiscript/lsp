/// Some constant.
export const X = 10;

/// This function is private.
private fn cant_touch_this() {

}

/// This function is exported.
fn hello() {
    print("hello");
}

fn asd() {
    let i = 0;
    for i in range(1, 2)  {
        i += 1;
    }
}