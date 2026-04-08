

# secp256k1-sys 自定义 Patch 记录

本 crate 基于上游 [rust-bitcoin/rust-secp256k1](https://github.com/rust-bitcoin/rust-secp256k1/) 的 `secp256k1-sys`，在其基础上做了以下自定义修改。

**当前基线版本**：`0.13.0`（2026-04 升级，从 0.11.0）

## Patch 清单

### 1. `PublicKey::serialize` 改为 `pub`

- **文件**：`src/lib.rs`
- **改动**：`fn serialize(&self) -> [u8; 33]` → `pub fn serialize(&self) -> [u8; 33]`
- **原因**：上游将该方法限制为 crate-private，我们需要在外部直接序列化 `PublicKey`。

### 2. 新增 `secp256k1_ec_seckey_invert_ct` / `secp256k1_ec_seckey_invert_vt`

- **文件**：
  - `src/lib.rs`（FFI 声明）
  - `depend/secp256k1/include/secp256k1.h`（C 头文件声明）
  - `depend/secp256k1/src/secp256k1.c`（C 实现）
- **改动**：新增两个函数，对私钥做模曲线阶的乘法逆元。`_ct` 为常量时间（使用 `scalar_inverse`），`_vt` 为变量时间（使用 `scalar_inverse_var`）。
- **原因**：上游 libsecp256k1 不提供私钥逆元 API，我们的 MPC 协议需要此操作。

### 3. `ec_pubkey_combine` 允许求和为无穷远点

- **文件**：`depend/secp256k1/src/secp256k1.c`
- **改动**：原版在求和结果为无穷远点（identity）时返回 0（失败），改为返回 1（成功）。
- **原因**：MPC 签名中间步骤可能出现公钥相加为零的合法情况，原版行为会误报错误。

### 4. 绕过 MuSig2 模块

- **文件**：`build.rs`（移除 `ENABLE_MODULE_MUSIG`）、`src/lib.rs`（移除所有 `Musig*` 类型和 FFI 函数）
- **原因**：当前不需要 Schnorr 多签，减少编译时间和攻击面。
- **注意**：vendored C 源码中的 musig 文件仍保留，只是不编译。如需启用，在 `build.rs` 加回 `.define("ENABLE_MODULE_MUSIG", Some("1"))` 并在 `lib.rs` 补回类型和 FFI 声明。

## 升级指南

从上游新版本升级时：

1. 下载新版 crate：`curl -sL "https://crates.io/api/v1/crates/secp256k1-sys/VERSION/download" | tar xzf -`
2. 替换 `depend/`、`src/`、`build.rs`、`Cargo.toml` 为新版文件
3. 按上述 Patch 清单逐条重新应用：
   - `src/lib.rs`：`PublicKey::serialize` 改 `pub`；添加 `seckey_invert_ct/vt` FFI 声明；`ec_pubkey_combine` 加注释；删除 MuSig2 类型和 FFI
   - `build.rs`：删除 `ENABLE_MODULE_MUSIG` 行
   - `depend/secp256k1/include/secp256k1.h`：添加 `invert_ct/vt` 函数声明
   - `depend/secp256k1/src/secp256k1.c`：添加 `invert_ct/vt` 实现；修改 `ec_pubkey_combine` 的 infinity 检查
4. **注意 symbol prefix**：每个大版本的符号前缀不同（如 `rustsecp256k1_v0_13_`），需要同步更新所有 C 函数名和 Rust `link_name`
5. 运行 `cargo build` 验证编译通过
