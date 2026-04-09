# secp256k1-sys (svarog fork)

基于 crates.io 上的 `secp256k1-sys 0.13.0`, 修改后通过 FFI 暴露内部的
group/scalar/field 运算.

## 动机

上游 crate 只暴露高层 API (`PublicKey`, `SecretKey` 等). v0.13 的 C 代码在遇到
全零 `PublicKey` (单位元) 时会 panic, 导致把 `PublicKey` 当作任意曲线点使用的代数
运算全部崩溃. 我们需要直接操作内部的 `ge`/`gej`/`scalar` 类型及其算术.

## 改动内容

### C 层

**`depend/secp256k1/src/svarog_algebra_impl.h`** (新文件)

32 个非 static 的包装函数, 统一使用 `svarog_` 前缀, 将链接器不可见的 `static`
内部函数暴露出来. 按类别:

- 群元素操作: `gej_set_infinity`, `gej_is_infinity`, `ge_is_infinity`,
  `ge_set_gej_var`, `gej_set_ge`, `ge_set_xy`, `gej_neg`, `gej_add_var`,
  `gej_add_ge_var`, `gej_double_var`, `ge_to_storage`, `ge_from_storage`
- gej 高层便利函数: `gej_eq_var`, `ecmult_const_gej`,
  `gej_to_storage`, `gej_from_storage`, `gej_serialize`, `gej_parse`
- 标量乘法: `ecmult_const`, `ecmult_gen`
- 域元素操作: `fe_set_b32_limit`, `fe_get_b32`, `fe_normalize_var`
- 标量操作: `scalar_set_b32`, `scalar_get_b32`, `scalar_negate`,
  `scalar_add`, `scalar_mul`, `scalar_inverse_var`, `scalar_is_zero`,
  `scalar_eq`
- 工具函数: `seckey_inverse`

**`depend/secp256k1/src/secp256k1.c`** (加了一行)

```c
#include "svarog_algebra_impl.h"   // 追加在文件末尾 (第 776 行)
```

头文件在编译单元末尾引入, 此时所有内部 `static` 函数已定义, 包装函数可直接调用.

### Rust 层

**`src/lib.rs`** (在 `#[cfg(test)]` 之前追加)

- FFI 结构体定义: `Fe` (40B), `FeStorage` (32B), `Ge` (88B),
  `Gej` (128B), `GeStorage` (64B), `CScalar` (32B)
- `Gej::new_infinity()` 便利构造函数
- `extern "C"` 块, 声明全部 32 个 `svarog_*` 函数

除上述追加内容外, 未删改上游的任何代码.