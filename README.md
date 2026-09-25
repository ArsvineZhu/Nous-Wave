# Nous Wave

Nous Wave 是持续 Subject 的认知系统。长期目标设计、设计决定和研究资料由 [Heptalogos-Devs/Architecture-Vault](https://github.com/Heptalogos-Devs/Architecture-Vault) 维护；本仓库保存当前实现、可执行规格、实施计划和验证事实。

## 文档

- [当前实现状态](docs/Nous_Wave/CURRENT_STATE.md)
- [仓库文档入口](docs/Nous_Wave/README.md)
- [当前实施计划](project/plans/active/README.md)
- [Nous Wave 目标设计 PDF](https://github.com/Heptalogos-Devs/Architecture-Vault/releases/download/nous-wave-target-design-latest/Nous_Wave_Target_Design.pdf)

当前代码处于架构重整前的可运行研究实现。代码中的数据类型和算法不会因为已经存在而自动成为目标设计。

## 开发

Rust 工具链由 `rust-toolchain.toml` 固定。常用入口：

```text
just fmt
just check
just lint
just test
just verify
```

当前物理实现包含 TypeScript Host、Rust Kernel、PostgreSQL Authority、对象存储以及词项、稠密和拓扑服务投影。具体状态见 `docs/Nous_Wave/CURRENT_STATE.md`。
