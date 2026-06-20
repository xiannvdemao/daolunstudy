# Episode: DELETE 性能优化 — 从 206 到 64,896 QPS

## 发生了什么
v2.9.0 DELETE 只有 206 QPS，远低于 E-09 地板 (10,000)

## 根因分析
- 逐条 DELETE 导致 B+Tree 索引频繁更新
- WAL 同步写盘拖慢每次操作

## 优化方案
1. 批量提交：合并多个 DELETE 为一次事务
2. WAL 异步刷盘：Group Commit 减少 I/O
3. 索引批量更新：减少 B+Tree 维护开销

## 结果
DELETE QPS: 206 → 64,896 (315x)
达到 E-09 地板 ✅

## 教训
- Pattern: "批量提交优先于逐条优化"
- 回归检测阈值 Δ > 20% = FAIL
