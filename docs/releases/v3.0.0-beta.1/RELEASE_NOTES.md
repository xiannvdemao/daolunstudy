# SQLRustGo v3.0.0-beta.1

## 核心改进
- DELETE QPS: 206 → 64,896 (315x 提升)
- UPDATE QPS: 950 → 43,121 (45x 提升)

## Gate 验收
| Gate | 结果 |
|------|------|
| BP1 静态 | ✅ |
| BP2 集成 | ✅ (TPC-H 22/22, 覆盖率 78%) |
| BP3 风险 | E-09: R-05 豁免 (cargo audit 已知漏洞) |

## 已知问题
- R-05: serde_json CVE-XXXX, 上游未修复

## E-09 Floor 验证
- DELETE ≥ 10,000 ✅ (64,896)
- UPDATE ≥ 10,000 ✅ (43,121)
