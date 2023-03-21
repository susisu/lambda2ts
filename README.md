# lambda2ts

A PoC compiler that translates untyped lambda calculus into TypeScript types.

## Background

Recently, a technique has been discovered that enables the use of first-class type-level functions in TypeScript (see [gvergnaud/hotscript](https://github.com/gvergnaud/hotscript)). For example, we can write and use first-class type-level functions as follows:

``` typescript
// Fun is the interface that represents first-class type-level functions.
interface Fun {
  arg: unknown;
  ret: unknown;
}

// App<F, X> applies a function F to an argument X.
type App<F, X> = F extends Fun ? (F & { arg: X })["ret"] : never;

// Functions are defined by extending the Fun interface.
// The argument can be accessed via this["arg"].
interface IsNumber extends Fun {
  ret: this["arg"] extends number ? true : false;
}

// Apply functions using App<F, X>.
// Functions can also be composed using high-order functions.
type FilterSet<F, X> = X extends unknown ? (App<F, X> extends true ? T : never) : never;
type MyType = FilterSet<IsNumber, "foo" | 42 | true | null | 0 | undefined>; // = 42 | 0
```

This idea enables us to compile [untyped lambda calculus](https://en.wikipedia.org/wiki/Lambda_calculus) to TypeScript types more directly than before. Therefore, I have developed a PoC compiler.

## License

[MIT License](http://opensource.org/licenses/mit-license.php)

## Author

Susisu ([GitHub](https://github.com/susisu), [Twitter](https://twitter.com/susisu2413))
