# svarog-secp256k1-sys

本包是 `secp256k1-sys 0.13.0` 的 Svarog fork，自身版本从 `0.1.0` 开始。
上游项目为 [rust-bitcoin/rust-secp256k1](https://github.com/rust-bitcoin/rust-secp256k1/)。
目录保留 `secp256k1-sys` 名称，crates.io 包名和 Rust 库名分别为
`svarog-secp256k1-sys` 和 `svarog_secp256k1_sys`。

## 扩展接口

此 fork 通过 FFI 暴露内部的群、标量和域运算，以支持包含单位元的曲线代数。
`depend/secp256k1/src/svarog_algebra_impl.h` 包装内部静态函数，
由 `secp256k1.c` 在编译单元末尾包含；Rust 声明位于 `src/lib.rs`。
现有扩展包含 32 个 `svarog_*` 函数以及 `Fe`、`FeStorage`、`Ge`、`Gej`、
`GeStorage` 和 `CScalar` 结构体。

这些结构体直接对应上游内部布局，升级上游时必须重新核对布局与编译配置。
目前 Rust 声明使用 5×52 位域表示与 4×64 位标量表示。
发布检查在宿主平台执行，不代表所有交叉编译目标都已经验证。

## 符号隔离

本包使用独立的 `links = "svarog_secp256k1_v0_1"` 和同名前缀的静态库。
`build.rs` 扫描 vendored C 源码，在 `OUT_DIR` 生成命名空间头文件和编译入口，
将上游 `rustsecp256k1_v0_13_*` 标识符映射到 `svarog_secp256k1_v0_1_*`。
扩展函数的链接符号为 `svarog_secp256k1_v0_1_svarog_*`，Rust 调用名仍为 `svarog_*`。
Rust 导出的回调也使用相同的 fork 前缀。

该过程不改写已检入的 vendored 文件，也不在构建时下载代码。
`tests/coexistence.rs` 在同一程序中调用官方 `0.13.0` 和本 fork，验证两者能共同链接并运行。
FFI 对象和上下文分别归各自的库所有，不应跨库传递。

## 更新上游

`Cargo.upstream.toml` 和 `README.orig.md` 保留上游资料，实际构建使用 `Cargo.toml`。
`vendor-libsecp.sh` 默认保留上游 `0_13` 命名空间，并重新加入 Svarog 扩展头文件。
更新时需指定明确的上游提交，检查所有补丁和 FFI 布局，再运行完整测试与共存测试。
若改变上游 ABI 基线或 fork 的不兼容版本，需同步维护构建映射、Rust 链接名和 `links`。

## 许可证

Rust sys 层保留上游的 `CC0-1.0` 声明，全文见 `LICENSE`。
Vendored libsecp256k1 的许可证见 `depend/secp256k1/COPYING`。
其他随附数据的许可证文件也保留在原目录中。
