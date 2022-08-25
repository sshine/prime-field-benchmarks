---
marp: true
theme: uncover
---

# Efficient prime-field arithmetic in Rust

`Simon Shine <simon@neptune.cash>`

Slides available on:

https://github.com/sshine/prime-field-benchmarks/

---

# Who am I?

I'm Simon.

I work at Triton Software 🇨🇭, and I make zero-knowledge cryptography for a privacy-focused blockchain called Neptune: https://neptune.cash/

![width:200px](https://i.imgur.com/K5Y6U0u.jpg)

---

# Who am I?

I'm Simon.

In my spare time, I study Chinese and practice<br>time travel (mostly forwards, in kayaks).

![](https://i.imgur.com/KKnqYut.png)

---

# Disclaimer

I'm not a mathematician. Don't hesitate to add to what I say. Also, I didn't invent anything here. All of this is re-discovered. I try to give credit.

---

## *tl;dr*

I $+$ and $\times$ mod $p$ faster using a combination<br>of hacks, tricks, and knowledge of $p$.

---

## Field arithmetic

A **field** is a set $S$ with $+$ and $\times$ where

- $+$ and $\times$ associate, commute and distribute
- $\exists0\in S$ so for all $a\in S$, $0 + a = a$ and $a + 0 = a$.
- $\exists1\in S$ so for all $a\in S$, $1 \times a = a$ and $a \times 1 = a$.
- $\forall a\in S$, $\exists b\in S$ so $a + b = b + a = 0$
- $\forall a\in S$, $a\neq 0$, $\exists b\in S$ so $a\times b = b\times a = 1$
- $0\neq 1$

---

## Field arithmetic

It just means you can $+, -, \times, \div$ as you'd expect.

Reals and rationals are examples of infinite fields.

Most people call this "**math**".

---

## Finite-field arithmetic

An example of an (efficient) **finite field** is<br>`u64` with overflow (aka $\text{GF}(2^{64})$).

Most programmers call this "**math**".

---
## Prime-field arithmetic

A **prime field** $\mathbb{F}_p$ is a finite field that overflows at $p$. Or said with Rust code,

```rust
use std::ops::{Add, Mul, Sub, Div};
use num_traits::{Zero, One};

pub trait PrimeField:
    Zero + One + Add + Mul + ModReduce + Sub + Div + Eq {}

pub trait ModReduce {
    #[must_use]
    fn mod_reduce(product: u128) -> u64;
}
```

---

## Inefficient prime-field arithmetic in Rust

```rust
// 2^64 - 2^32 + 1
pub const P64: u64 = 0xffff_ffff_0000_0001;
pub const P128: u128 = 0xffff_ffff_0000_0001;

pub fn add(x: u64, y: u64) -> u64 {
    let sum: u128 = x as u128 + y as u128;
    (sum % P128) as u64
}

pub fn mul(x: u64, y: u64) -> u64 {
    let product: u128 = x as u128 * y as u128;
    (product % P128) as u64
}
```

---

## Efficient prime-field $+$

```rust
pub fn add_fast(x: u64, y: u64) -> u64 {
    let mut sum: u128 = x as u128 + y as u128;
    if sum > P128 {
        sum -= P128;
    }
    sum as u64
}
```

<!-- How does this look like in assembler, and what's the timing difference between the two branches? -->

---

## Efficient prime-field $\times$

*(for $p = 2^{64} - 2^{32} + 1$)*

Credit: cp4space.hatsya.com's blog post:<br> [An efficient prime for number-theoretic transforms](https://cp4space.hatsya.com/2021/09/01/an-efficient-prime-for-number-theoretic-transforms/)

- Elements in $\mathbb{F}_p$ where $p = {2^{64} - 2^{32} + 1}$ fit nicely inside a 64-bit machine word, and "mod $p$" is possible without multiplication or division.

---

## Efficient prime-field $\times$

*(for $p = 2^{64} - 2^{32} + 1$)*

Any non-negative integer less than $2^{159}$ can be written as $A2^{96} + B2^{64} + C$ where A is a 63-bit integer, B is a 32-bit integer, and C is a 64-bit integer. 

---

## Efficient prime-field $\times$

*(for $p = 2^{64} - 2^{32} + 1$)*

Since $2^{96}$ is congruent to $−1$ modulo $p$, this can be rewritten as $B2^{64} + (C - A)$. If $A > C$, $B2^{64}$ could underflow, in which case we can add $p$, resulting in a 96-bit integer.

---

## Efficient prime-field $\times$

*(for $p = 2^{64} - 2^{32} + 1$)*

To reduce this to a 64-bit integer, $2^{64}$ is congruent to $2^{32} - 1$, so we can multiply $B$ by $2^{32}$ using a binary shift and a subtraction, and then add it to the result. We might encounter an overflow, but we can correct for that by subtracting $p$.

---

## Efficient prime-field $\times$

*(for $p = 2^{64} - 2^{32} + 1$)*

See the [source code](https://github.com/sshine/prime-field-benchmarks/blob/main/src/lib.rs). How fast is it, then?

---

## `cargo criterion`

11th Gen Intel(R) Core(TM) i7-11700 @ 2.50GHz

```
add/baseline/1000       time:   [0.0000 ps 0.0000 ps 0.0000 ps]                                        
add/mod/1000            time:   [2.5268 µs 2.5290 µs 2.5317 µs]                           
add/fast/1000           time:   [1.0189 µs 1.0195 µs 1.0202 µs]                            
add/winterfell/1000     time:   [1.0564 µs 1.0573 µs 1.0581 µs]                                  

mul/baseline/1000       time:   [0.0000 ps 0.0000 ps 0.0000 ps]                                        
mul/mod/1000            time:   [2.5743 µs 2.5756 µs 2.5770 µs]                           
mul/reduce159/1000      time:   [1.6030 µs 1.6041 µs 1.6052 µs]                                 
mul/reduce_montgomery/1000                                                                              
                        time:   [1.3197 µs 1.3206 µs 1.3215 µs]
```

---

## `cargo criterion`

MacBook Pro M1 2021

```
add/mod/1000            time:   [7.2183 µs 7.2243 µs 7.2303 µs]
add/fast/1000           time:   [1.3538 µs 1.3570 µs 1.3602 µs]
add/winterfell/1000     time:   [1.2872 µs 1.2876 µs 1.2880 µs]
mul/mod/1000            time:   [13.447 µs 13.467 µs 13.487 µs]
mul/reduce159/1000      time:   [1.0562 µs 1.0569 µs 1.0576 µs]
mul/reduce_montgomery/1000
                        time:   [975.32 ns 975.60 ns 975.94 ns]
```

---

## Godbolt!

See the disassembly for [x86_64][godbolt-1] and [aarch64][godbolt-2].

---

## That was $+$ and $\times$... what else?

For prime fields and uni-/multivariate polynomials:

- `AddAssign` (`+=`), `MulAssign` (`*=`), when possible
- `.mod_pow()` with halving strategy ($x^{2n} = x^n\cdot x^n$)
- Generally, don't divide, multiply by inverses.
- Specialize `.square()`.
- Only re-allocate coefficients when `rhs.len() > lhs.len()`.
- Short-circuit polynomial multiplication when operand is $f(x) = 0$ or $f(x) = 1$.

---

## EOF

See Alan Szepieniec's STARK anatomy tutorial: https://neptune.cash/learn/stark-anatomy/

[godbolt-1]: https://godbolt.org/#z:OYLghAFBqd5QCxAYwPYBMCmBRdBLAF1QCcAaPECAMzwBtMA7AQwFtMQByARg9KtQYEAysib0QXACx8BBAKoBnTAAUAHpwAMvAFYTStJg1DEArgoKkl9ZATwDKjdAGFUtEywZ7HAGTwNMAHLuAEaYxCDSAA6oCoR2DC5uHnrRsbYCvv5BLKHh0laYNvFCBEzEBInunlyWmNbpDCVlBJmBIWERlqXllck15s2t2bkRAJSWqCbEyOwcAPRzANQATACkAKzYAGySiwC0KxvYAMzLi6vLAEKLXKsaAIKRJsGLaAzmi8o7IIsmO%2BfHAAiiw0qio4KoAH0IVCNHCNJD4bdjpc7o9nq8BB9lFxlgAOH4mXF4gHA0Ew6EQxHw6kaZGoh5op4vKgMRZMdDoCDqX47UiLACehJ2o32q2O2F5u1WAHYGfdForFvQCIsFO5CcTSYtVOyFL8tRdrgK9Qb8eL5UrFhB1Sxzst1p9iaKmPq/pI0bLAZ6HszFqz2Zzoa6CNzhdJBeHRXtxZL3ec5WirSrFiwTKrbZr8drda6zSSjYLTUTzSik0q8FQ1e4AZKcdnZZarUrbWKgU7S03FV7y4rW3n3Z6Zd7Gb6MQGOehIQB3PwEMJUOq0MNS/lCqXR2NShNdxYLdn264vcXApj7a2Rc/BUa95WYVXcmqvLiik86gB0qAAbgvaKhZ0YkLqsEEBfLsBwCjeZYPMm96Bto2oaCYpzvtOxBMJEkR%2BMAQHPBAyBcMWpxQbuqhcKh6GYdhuEgRyCEDiKQ4jvcTLjmyabLjy7prlGbZxv8ja3imkTEBgJg2FmBbtrmbpagAVEWA7Ehat4QCJYk2Pajr1niLpujsTE%2BuiLLsSYtCQsQmDoOJmBcOsACcK7cZGG58dugkwUqwmidZEn5jmxbyYpsmdrelm%2BbZDlqT54kEDejLDkZfoBhxFlWTZkIsLIwCoGwxACk5fIue6m4Su5iaeYq3kaQQkkBUp2YKSaDV4iplWLOFGVZYIOV5QV6m%2BXFhmjvcCxLPcCi2pgiwEAgTCqrqbyxOY%2BqoFW/BTIspx7MEhCLF%2BYgmJgCg/Ew/LBPyyD8ugIBomNd37gcZ5vKUfj6qcd4hmqeDAAwlZ4KIgiLLtBAKKQD1LAc6CYoITBvVtZxZR8sS/f9gOqiDCjvhDEM6oseD6sEokANaMPjgioJixCWQo0QMPgRj7Ydx2mgoCD/myoR/tOaJLaq3gAPIAOrYAASpCACy9xCAA0uGiFglSMJtSxjLLMcGyXH4tB%2BJgEBiNOTACgoN7rMxAadTMdmOVxzpufGHkKl5cFMMe0nFgZ0FO1VcHINDb7crWW4ih7Hpe7BqrINq%2BHQwOxGhyrEeLP77Z%2B0HZWnCrt77mgLBPPO7IvFDFrA4UTBmNN0MsEbpeLMAlnzWEM1zWyrv8i36D0Is0TzoIeBiL89O/v%2BQlwRABB5xo/IE5CJhD8Qr7tq7n4/sQVDc9RwEQOgJGj6qE%2BRIRb4HxoFEYVhgFb4LIvi1LsuLApEAz3PWDEKHoy7yNVo57l%2BfTVHCk1ibA%2BgcZAJdZoE3xvqYAJgyiGHnFZRYDBUD70pi/YePN2opgPmcAOUdxROAIQjaMrws7tX3JOGamA85MzcCzQw0NLIECmGyWa00aZmQICXUIogK6LEwEwZACBFhrWbtNecNDq4mlCL8S8RBFj/ExuDchSwO5dx7owWwA9vwYL3taDhtALBQMhDohe2oD7kVMevf81FJzjzzssT%2B3sOrHU4WfKigE7HXzFpLaWMsH7WhnqY9%2B8UWKJRGhcDW6wtYMB1v4fWtBDbG1NsxPm/o2SW0wJlbKuUwgFVtviUq/FpQVWcfuIQmB2GYEXJZBgMxC46P9CQA8WAFDIGIHgSIDQRFVjYU02gG9GZ4DzvQNgsMGjYywXBVQtB6r6TDruFMqhhEB11LGYOkg9JSkTs7B8Z1%2BGL2BDMleGDbGcm5LMghRDiJkOcSmN2p53EXxwlvM86yM6OKeZvPC00GKbNuUnCAZBXiHJ1AgE5a9BkvLwteHZipiBfMvnhJCKE0Ln2%2BSBKOcdHHv2GvcDg4xaCcHWLwTwHAtCkFQJwUWZgMyTGmNNSJPBSC1XJQS8YxMQDrCnkSjgkhSWaF4FSjgvATpT1ZVocYcBYBIFzlhegZAKD4V/nQDoxAuBbGOFPGghiwgnQgMEQVpBdrMHypwZlucxkEAFrEoUbLSBYGrkYcQ9r8CWSKD%2BE69rMCqEKOmWYzK5x1CNTrImZQBQuCwEaggHSWDmoJXwAwwAFAADU8CYGnALSIjB40yEECIMQ7ApB5vkEoNQRrdA1AMEYEAphzD6DwMEE6kBxioC6fEL1ewnAiK6XsegP5aAnmOKKuohQGgOHpr0aopAfC62GB0SQ9lSCpDiAIKdKQYiroYEMdoeQl0FCKAIJoPRXBVD0Aehox6Whzt3REfd3QKinr6F0QYN6cgLvsuMBQ9KZh6BjZU%2BNhLiUCvtcKutBAo7kU1e%2BDQ1oaX1sWLgQgzSmX8hcHnVVb8mWjF4BK9lpBOXcv0JwflpAyUUuFaKkA4rBUf2IxwZYIGKOcFw7R8Yq9Yj2EkEAA%3D%3D%3D
[godbolt-2]: https://godbolt.org/#z:OYLghAFBqd5QCxAYwPYBMCmBRdBLAF1QCcAaPECAMzwBtMA7AQwFtMQByARg9KtQYEAysib0QXACx8BBAKoBnTAAUAHpwAMvAFYTStJg1DEArgoKkl9ZATwDKjdAGFUtEywZ7HAGTwNMAHLuAEaYxCAAHBqkAA6oCoR2DC5uHnpxCbYCvv5BLKHhUZaY1lkMQgRMxAQp7p5cxaVJFVUEOYEhYXrmrbVpDT3V7XkFEgCUlqgmxMjsHAD08wDUAEwApACs2ABskksAtKub2ADMK0trKwBCS1xrGgCCMSbBS2gM5kvKuyBLJrsXE4AESWGlUVAhVAA%2BpDoRp4RooQi7icrvcni83gJPsouCsIr8THiIoCQWDYTDIUiEdSNCi0Y90c9XlQGEsmOh0BB1H9dqQlgBPQm7MYHNYnbC8vZrADsDIeS0VS3oBCWCnchOJpKWqnZCj%2BWsuNwFeoN%2BPF8qVSwg6pYFxWGy%2BxNFTH1/0k6NlQM9j2ZS1Z7M5MNdBG5wukgvDov24sl7oucvRVpVSxYJlVts1%2BO1utdZpJRsFpqJ5tRSaVeCoavcgMluOzsstVqVtrFwKdpabiq95cVrbz7s9Mu9jN9mIDHPQUIA7n4CGEqCVaGGpfyhVLo7GpQmu0tFuz7TdXuKQUwDtaYufgmNe8rMKruQ03lxRSedQA6VAANwXtFQs6MKF1WCCBvj2Q4BRvMtHmTe9A20bUNBMM532nYgmBiGI/GAICXggZAuGLM4oN3VQuFQ9DMOw3CQI5BCBxFIcRweJlxzZNNlx5d01yjNs4wBRtbxTGJiAwEwbCzAt21zN0tQAKiLAdiQtW8IBEsSbHtR16wiF03V2JifQxFl2JMWgoWITB0HEzAuA2ABOFduMjDc%2BO3QSYKVYTROsiT8xzYt5MU2TO1vSzfNshy1J88SCBvRlhyMv0Aw4iyrJsqEWFkYBUDYYgBScvkXPdTcJXcxNPMVbyNIISSAqU7MFJNBqIhUyqlnCjKssEHK8oK9TfLiwzRweRZlgeBRbUwJYCAQJhVV1d4EnMfVUCrfhpiWM59mCQgli/MQTEwBRfiYflgn5ZB%2BXQEB0TGu790OM93kqPx9TOO8QzVPBgAYSs8FEQQll2ggFFIB7lkOdAsUEJg3q284ss%2BBJfv%2BwHVRBhR3whiGdSWPB9WCUSAGtGHxwRUCxYhLIUOIGHwIx9sO47TQUBB/zZUI/2ndEltVbwAHkAHVsAAJShABZB4hAAaXDRDwSpWE2pYxkVhOTYrj8Wg/EwCAxGnJgBQUG8NmYgNOtmOzHK4503PjDyFS8uCmGPaTiwM6CnaquDkGht9uVrLcRQ9j0vdg1VkG1fDoYHYjQ5ViOln99s/aDsqzhV299zQFhnnndlXihi1gcwUQzGm6GWCN0ulmASz5rCGa5rZV3%2BRb9B6CWOJ50EPAxD%2Benf3/IS4IgAg8%2BifGFChEwh%2BIV921dz8f2IKhueo4CIHQEjR9VCeYkIt8D40CiMKwwCt8FkXxal2WlgUiACdn%2BfQ7GXeRqtHPcvz6ao4U9YWwPqHGQCXWaBNp51xMFUQw84rJLAYKgfelM55YDXtzPeM087nADlHcUTh8EI2jG8LO7V9yThmpgPOTM3As0MNDSyBBphslmtNGmZkCAl1COXJQSxMBMGQAgJYa1m7TXnNQ6uJpQh/EvEQJYAJMbgzIcsDuXce6MFsAPb8w8ebtRTBAdhtALDTyhNohe2oD7kTMevf81FJzj2wR/b2HVjocLPlRQC9jr5i0ltLGWD9rTPzMW/eKLFEojUuBrDYWsGA638PrWghtjam2Ynzf0bJLaYEytlXKYQCq23xKVfi0oKrOP3EITAbDMCLksgwWYhdtH%2BhIAeLAChkDEDwDEMowiqysKabQDejM8B53oGwWGZRsZ6LgqoWg9V9Jh13CmVQQiA66ljMHSQekpSJ2dg%2BM6fDF4ghmSvHRdjOTclmfgwhxFSHOJTG7U87iL44S3medZGcVhjCeZvPC00GKbNuUnAxl1Dk6gQCc9BtjL54WvDsxUxBvnQpAkhFCaFz4/JAlHOOny37DQeBwCYtBOAbF4J4DgWhSCoE4KLMwGYpgzGmpEngpBarkoJRMYmIANjRCJRwSQpLNC8CpRwXgJ1oisq0BMOAsAkC5ywvQMgFB8I/zoF0YgXBtgnGiDQIxYQToQGCIK0gu1mD5U4My3OYyCAC1iUKNlpAsDVyMOIe1%2BBLI2DwD%2BE69rMCqDLumOYzK5wlCNTrImVQBQuCwEaggHSWDmoJXwAwwAFAADU8CYGnALGIjAE0yEECIMQ7ApD5vkEoNQRrdANAMEYEAphzD6DwMEE6kAJioC6Ukb1%2BwnDCK6fsegP5aAnhOAcfYlRiDABdlUQRux9hz2Jog6cDB%2B1%2BBMKofYv0TCipKGXMoDh6Z9HqKQHwusRhdAaBkRIAhD3pHiFehgwxOjhAGDuj1AgWjVBvS%2Bpo77x1tFPU%2B7of6v2WD/Y%2B/I56JgKHpbMPQsbKkJsJcSgV9rhX1oIFHcimr3waGtDShtSxcCEGaUy/kLg86quIPaE4L5eASvZaQTl3L9CcH5aQMlFLhWipAOKwV78WMcBWChzjnA6N8YmKvBI9hJBAA
