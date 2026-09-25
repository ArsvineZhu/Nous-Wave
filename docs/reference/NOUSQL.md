# NousQL 当前实现参考

本页描述 `apps/nous-core` 当前 parser/compiler 支持的 NousQL 子集。语义查询最终编译为 Protobuf `QueryExpr`；查询算法与检索通道由 Kernel/Serving 实现。

## 当前语法

- 原子：引号包围的文本、`#concept`、`*`，以及 `@e`、`@tag`、`@anchor`、`@r`、`@object`、`@ref` selector。
- 组合：显式 `&&`、`||` 和括号；`&&` 优先于 `||`。
- 软偏好：`+atom`、`-atom` 和 `+recent`、`-recent`。
- 指令：`$memory`、`$evidence`、`$resource`、`$effort`、`$limit`、`$source`、`$modality`、`$memoryClass`、`$evidenceClass`、`$authority`、`$current`、`$diagnostics`、`$explore`、`$materialize`、`$exclude` 和 `$time`。

单个软偏好操作数目前也接受冗余括号；canonical form 会省略这层括号。

实体 selector 可列多个参与者；绑定后按 LexicalRef 排序，重复规范身份会报错。名称必须由 Kernel identity resolver 唯一绑定；解析歧义不会退化为向量猜测。`@object` 接收不透明 Host 引用，`@ref` 接收 LexicalRef。

`$time` 支持 `occurred`、`observed`、`valid` 三条时间轴，时间点必须带 ISO-8601 offset；`within` 不能与 `from/to/at` 混用。查询指令在同一表达式节点内去重，跨 scope 的 modifier 不自动搬移。

## 当前限制

- 单个查询上限为 32 KiB、2048 个 token、16 层嵌套；`$limit` 范围是 1–2048。
- `persona`、`relation` 和 `rel` 当前返回 `FAILED_PRECONDITION`，对应认知领域没有当前服务实现。
- 此接口不提供物理算法 selector；它只表达 typed query intent 和约束。

精确 parser/compiler、生成协议及当前行为测试见 [`apps/nous-core/src/nousql`](../../apps/nous-core/src/nousql)、[`proto/nous/wave/v1alpha1`](../../proto/nous/wave/v1alpha1) 和 [`apps/nous-core/tests/nousql.test.ts`](../../apps/nous-core/tests/nousql.test.ts)。
