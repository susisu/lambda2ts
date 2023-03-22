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

## Demo

- [Input](https://github.com/susisu/lambda2ts/blob/main/examples/factorial.ml)
- [Output](https://www.typescriptlang.org/play?#code/PTAEEEDtQUwDwIYFsAOAbGoUCcD2BzbZUAFwAsETQBjBNagVzUpgGdQAzBak3bASzqsAUCFBkSJFKwBcIfP3IMARgDpquJMFYNW-HcGZJlAEwQAmEiOEkAnikwAVNiQBi3XgLqgAvKEe4AHIMxjDYADzgKCjh7jx8gmgANKCueEjBoREArAB8+QDcomCgoAB6APzC1WIAtPWgAKok-GiK-GycfDRkMNQA1vyQ+KDYbExWoPW1wkMkYVzUmGmamcph4YGwcPOQJuyQIevYudu7+6kM0ADewqVjJDKp6WthACQAjJsp5PoA2gAiBDYfAAgC6uSKAF9ZpB5thFssXkd3l8tvBzgcUdgUq5ThiYHt2K4rqBbvcYI8INFAt9Uj8yP8gSDwZDhDCbPZMFEULTAriUgANU5+HmBT501xClJ-CFFOwOam8iX8+mgQUpADCZ0JFyu-UguAA7pBZSK7qBNYCMMNyOCdUTQIELaUKuqXaAnmKVQKlbEhbkZfrDSaUqpw5q5dUFU4gtjNiL-HGshKeZFonSAJKQagnGUQtkxpOvbASxyJxwOvWQA3G01g0BuxzWwn4O0Np6QGAANzCRVh8MRoGzuarxNJ5NGlKev1YgOBoIbBMdwbrssboD+q9DoHDqln85ZYI7oC7vew0OqxSmDQA4oSwiwTDRcCZMNMBwtuMsJ6AF09t0gAopypQDQBhIs0yldVE1cMdLmgN0AAo4IAMjJP8QSeQVwIASkBB57U7Hs+0-BFv2HZ9lwuEkbhAp5MxMCUD2ZRdTggrlKIlOBEzgIo5i-JZLVwSBWCoajxzoh4nk1ESxOYxk51Y1lwLIodZNEkhuPxHZdUkjDpOEzS3nMcI4AZJkFxUjjFQ0+TTPM0BbF4-i4UE5Z+DgN5uwABng2iDOnVJPO8nyFMso92LUijXBC3ztP839DNirz4ociylKsiFVKLFLQpMsyUmc3w-TTRyeKKtkBPIoS8u7RKpKCurwsyyLVOqodmq+DgdMxBDAqpZrTI4DLDzYnLOKG8IRtAHiSrTGaoLinyUh4qq3JqjzUo+BqBqeOqPhasbrOi2q4sOr45ok-rrno4LtoKxyWKyqLcvOx7KvmjMytWwMnPWwcYuWvzroC27kuBo7lOymEOqB1KfLC7rer0m67rqxGCpm562psrbQrC4bfq+mJFozA7foB9z7t28GmpCr4cfGvH7olHqSfTGI6pSHqUiWhGVs4QpTswAB5Ltabu8WYChl72o2odpbZlHHTBqWuyx0bodezilaJ2bEwW37XMBoSAFkmElwyLbQWXcZF0AbYlBAVZopKgqd0yEC1uXYYVijPfCb3QGUV39LpqknYAZiDlJlB9+24fNpg3hj4P46Fq2PZTgAWWOQ55hPmc5RUnbz9PC4Njm02DtMM56v6eJN6mAAUxiY+rQfdqk25gDu7eLpPMF7-vkaznv2+8zXSEU46YYdkep-1-Aw7RwzF+7VPppSfAi5OosN63mbd-EQ2MzIPmMxPhvm820AR-Hp5F8Z2ftfl03h8nr5IFXtX18n0ykA97zyHvfAB4QgGZy7o1Cefct6QOxq-X2JdP5wJjgg4mopvrYJiJAtMG8eb5EvjEOyJBKYpEYoWTijhsAMEwH4Uht9OpCEwNAvaqQWEDxOqA9waBWAy0ur-bu+1OHpRnhFYuuVRGFX+iVWwTCKKZlYAALTCLgR+w4VFqK4fPIsSjVF4AlD-auGY8EZlIbiFhRD-C0JgFTO+cRPCJG8hoxxCQ6BTxfhI7h-taoeHcbbTeY82ERxEfELwgTp5Mx8R-Dh4TnGb31sYkJ6N-ERKnjHGakDom6MmmkhJR8UiQJOCYmIaZ9FqKKX9PMpUMxkxiCPKpf00xphtlUlIJxhZSPiR4jgHMUqWJ6YE+xzChmuPyR4rxrVJF5KGUYs+ZTyYTNtlkv60s2RAA) (`cargo run < ./examples/factorial.ml`)

## License

[MIT License](http://opensource.org/licenses/mit-license.php)

## Author

Susisu ([GitHub](https://github.com/susisu), [Twitter](https://twitter.com/susisu2413))
