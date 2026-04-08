# typo_compiler 优化计划

## Context
项目已完成核心功能（AI 纠错 + 编译器风格渲染 + 文件输入 + 分句）。现在进行一轮优化，提升并发性能、用户体验和代码质量。

## 实施顺序（按依赖关系排列）

### Phase 1：简单清理（无依赖，独立完成）

#### 1. 移除 lib.rs 中的 `mod cli`
- **文件:** `src/lib.rs`
- **改动:** 删除 `mod cli;` 这一行
- **原因:** cli 只被 main.rs 使用，放在 lib 里导致所有 cli 相关类型报 dead_code warnings

#### 2. 退出码
- **文件:** `src/main.rs`
- **改动:** `total_errors > 0` 时调用 `std::process::exit(1)`
- **原因:** 编译器/linter 发现错误应返回非零退出码，方便脚本集成和 CI

#### 3. 按行分句模式 (`-l`)
- **文件:** `src/cli.rs`
- **改动:**
  - Cli 结构体添加 `#[arg(short = 'l', long, conflicts_with = "divide")]` 的 `lines: bool` 字段
  - `resolve_input()` 中 `self.lines` 优先于 `self.divide` 判断
  - 新增 `split_lines()` 函数：按 `\n` 分割，跳过空行，记录行号
  - 添加单元测试

### Phase 2：用户体验

#### 4. 进度反馈
- **文件:** `src/main.rs`
- **改动:**
  - 多句模式下，每处理一句前向 stderr 输出 `[1/5] Checking...`（用 `\r` 覆盖）
  - 用 `std::io::IsTerminal` 判断 stderr 是否是终端，非终端时跳过进度输出
  - 处理完毕后用 `\r\x1b[2K` 清除进度行

### Phase 3：并发 + 限速

#### 5. 并发请求
- **文件:** `src/service.rs`, `src/main.rs`
- **service.rs 改动:**
  - `post()` 及内部方法返回值改为 `Result<String, Box<dyn Error + Send + Sync>>`
- **main.rs 改动:**
  - `Service` 用 `Arc` 包装
  - 用 `tokio::task::JoinSet` 并发发送请求
  - 收集结果后按原始索引排序，保证输出顺序不变
  - 单个请求失败时输出 warning 继续处理其余句子

#### 6. 限速（Rate Control）
- **文件:** `src/main.rs`
- **改动:**
  - 用 `tokio::sync::Semaphore` 限制并发数为 `provider.api_rate`
  - `api_rate <= 0` 时 clamp 为 1
  - 在 JoinSet spawn 前 acquire permit
- **注意:** 这是简单的并发数限制，不是精确的每分钟速率控制。对当前使用场景足够

### Phase 4：渲染优化

#### 7. 更精确的错误定位
- **文件:** `src/render.rs`
- **改动:**
  - 新增 `find_diff_span(original, fixed) -> Option<(usize, usize)>`：对比原句和修正句，找出第一个和最后一个不同的词，返回字节偏移范围
  - 修改 `render_sentence_level_error()`：如果能定位到具体差异词，只高亮差异部分 + 用 `^^^` 指向，列号也更精确
  - 修改 `render_word_choice_error()`：同理，用黄色高亮差异部分
  - 找不到差异时（如整句重写）fallback 到当前的整句高亮
  - 添加测试：单词变化、词首词尾、整句重写 fallback

## 涉及文件汇总

| 文件 | 改动项 |
|------|--------|
| `src/lib.rs` | 1 |
| `src/main.rs` | 2, 4, 5, 6 |
| `src/cli.rs` | 3 |
| `src/service.rs` | 5 |
| `src/render.rs` | 7 |

## 新增依赖
无。全部使用已有的 tokio（JoinSet, Semaphore）和标准库（Arc, IsTerminal）。

## 验证
1. `cargo test` — 全部通过
2. `cargo check` — 无 error，dead_code warnings 消除
3. 手动测试：
   - `cargo run -- "I has apple"` → 退出码为 1
   - `cargo run -- "I am happy"` → 退出码为 0
   - `cargo run -- -lf sample.txt` → 按行分句
   - `cargo run -- -df sample.txt` → 并发请求 + 进度反馈 + 限速
