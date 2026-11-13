# svarog-algebra

本 workspace 包含五个发布包：`curve-abstract`、`svarog-secp256k1-sys`、
`svarog-curve25519`、`svarog-secp256k1` 和 `svarog-lagrange`。
`classgroup` 与 `cl-playground` 独立保留，不参加 workspace 检查或发布。
所有 workspace 成员共用根目录的 `Cargo.lock` 和 `target`。

## 发布检查

本 workspace 使用 Rust 1.100.0，并在发布检查中使用 Clippy。
首次构建需要 C 编译器、`make` 和 `m4`；Cargo 会构建 vendored 原生依赖。
发布检查需要访问 crates.io，并要求待打包文件的修改已经提交。

```bash
scripts/prepublish.sh
```

脚本依次执行测试、将警告视为错误的 Clippy 检查、文档构建、打包和发布预演。
任一步失败都会停止；默认不跳过包验证，也不使用 `--allow-dirty`。
CI 使用同一脚本，实际发布由本地入口显式触发。

## 发布与重试

内部依赖同时声明 `path` 和 `version`：开发时使用本地源码，发布包使用注册表版本。
修改版本时，需要同步更新相关依赖要求，并更新、检查、提交根锁文件。
默认检查整个 workspace；仅发布部分包时，在脚本参数中列出包名。

```bash
scripts/publish.sh
scripts/publish.sh --execute
scripts/publish.sh --execute svarog-lagrange
```

`publish.sh` 默认执行预演，只有 `--execute` 才会在检查成功后上传。
Cargo 按依赖顺序发布，workspace 中的新依赖可以一起预演。
整批发布不是原子操作；中途失败后，核对 crates.io 的成功项，再指定剩余包重试。
若剩余包需要已上传的新依赖，等待注册表可解析该版本后再运行。
脚本不会自动改版本、提交代码或创建标签。

## sys fork

目录 `secp256k1-sys` 对应发布包 `svarog-secp256k1-sys`。
它基于上游 `secp256k1-sys 0.13.0`，提供额外的内部代数运算接口。
包名、原生链接标识和导出符号均与官方库隔离，详见该目录的 README。

## 许可证

四个自有发布包采用 `MIT OR Apache-2.0`，使用者可任选其一。
对应文本为 `LICENSE-MIT` 和 `LICENSE-APACHE`，各包内也包含完整副本。
此声明覆盖上述四个包；sys fork 及 vendored 代码保留各自的许可证文件。
