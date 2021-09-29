// This script tests string operations

print("hello");
print("this\nis \\ nice");      // escape sequences
print("0x40 hex is \x40");      // hex escape sequence
print("Unicode fun: \u2764");   // Unicode escape sequence
print("more fun: \U0001F603");  // Unicode escape sequence
print("foo" + " " + "bar");     // string building using strings
print("foo" < "bar");           // string comparison
print("foo" >= "bar");          // string comparison
print("the answer is " + 42);   // string building using non-string types

let s = "\u2764 hello, world! \U0001F603"; // string variable
print(`length=${s.len}`);       // should be 17

s[s.len-3] = '?';               // change the string
print(`Question: ${s}`);        // should print 'Question: hello, world?'

// Line continuation:
let s = "This is a long \
         string constructed using \
         line continuation";

// String interpolation
print(`One string: ${s}`);

// Multi-line literal string:
let s = `
          \U0001F603 This is a multi-line
                "string" with \t\x20\r\n
made using multi-line literal
      string syntax.
`;

print(s);

// Interpolation
let s = `This is interpolation ${
            let x = `within ${let y = "yet another level \
                                       of interpolation!"; y} interpolation`;
            x
} within literal string.`;

print(s);

print(">>> END <<<");
