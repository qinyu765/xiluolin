# T024 首页重构

## 任务目标

重构首页内容,整合"快速开始"和"语音输入成效"两个区域,优化历史记录展示,添加删除功能。对应任务跟踪表中的 T024 任务。

## 实际改动

### 后端改动

**src-tauri/src/data.rs**:
1. 新增 `delete_history_record` 方法:
   ```rust
   pub fn delete_history_record(&self, id: &str) -> rusqlite::Result<()> {
       self.connection.execute(
           "DELETE FROM history_records WHERE id = ?1",
           params![id],
       )?;
       Ok(())
   }
   ```

2. 新增 Tauri 命令:
   ```rust
   #[tauri::command]
   pub fn delete_history_record(app: tauri::AppHandle, id: String) -> Result<(), String> {
       let database = database_for_app(&app)?;
       database.initialize().map_err(|error| error.to_string())?;
       database
           .delete_history_record(&id)
           .map_err(|error| error.to_string())
   }
   ```

**src-tauri/src/lib.rs**:
- 在 `invoke_handler` 中注册 `data::delete_history_record` 命令

### 前端改动

**src/main.tsx**:
1. 移除技术标记:
   - 删除 "T015 主界面" 标记,卡片标题改为"快速开始"
   - 删除 "T009 历史与统计" 标记

2. 新增时间分组辅助函数:
   ```typescript
   function isSameDay(date1: Date, date2: Date): boolean
   function formatDateKey(date: Date): string
   function groupHistoryByDate(records: HistoryRecord[]): GroupedHistory
   ```

3. 新增删除历史记录处理函数:
   ```typescript
   async function handleDeleteHistoryRecord(id: string)
   ```

4. 使用组件替代内联 JSX:
   - 将"快速开始"卡片替换为 `<QuickStartCard />` 组件调用
   - 将"语音输入成效"卡片替换为 `<VoiceInputStatsCard />` 组件调用

**新增组件文件**:

**src/components/home/QuickStartCard.tsx**:
- 封装快速开始卡片的完整 UI 和交互逻辑
- Props 包含人格选择、录音控制、音频上传、结果展示等所有必要状态和回调
- 当前状态补充：该组件仍在代码中保留，但 `src/pages/HomePage.tsx` 中的 `<QuickStartCard />` 调用已被注释隐藏，首页当前不可见录音按钮、音频上传、原始识别文本和整理结果区域。

**src/components/home/VoiceInputStatsCard.tsx**:
- 封装语音输入成效卡片的完整 UI 和交互逻辑
- 包含统计卡片展示和历史记录时间分组展示
- 内部定义 `HistoryRecordItem` 子组件,负责单条历史记录的渲染

**src/components/home/index.ts**:
- 统一导出首页组件,简化导入路径

### UI 优化

1. **历史记录时间分组**:
   - 今天的记录显示在"今天"分组
   - 昨天的记录显示在"昨天"分组
   - 更早的记录按"X月X日"格式分组

2. **历史记录操作按钮**:
   - 移除"复制"文字标签和输出模式标签
   - 只保留复制图标按钮和删除图标按钮
   - 删除按钮悬停时显示红色,提升视觉反馈

## 为什么这么做

### 架构选择

1. **时间分段展示**:
   - 符合用户心智模型:"今天"、"昨天"、具体日期
   - 便于快速定位最近的记录
   - 减少视觉噪音

2. **添加删除功能**:
   - 用户需要清理无用记录
   - 避免历史记录无限增长
   - 提升用户控制感

3. **简化操作按钮**:
   - 移除"复制"文字和输出模式标签,只保留图标
   - 添加删除图标按钮
   - 减少视觉干扰,提升操作效率

4. **组件拆分**:
   - main.tsx 超过 2000 行,可维护性差
   - 将首页两个主要卡片抽取为独立组件
   - 每个组件职责单一,便于测试和复用
   - 通过 props 接口明确组件依赖

### UI 设计

1. **时间分段标题**:
   - 使用小标题区分不同时间段
   - 今天、昨天使用中文
   - 更早的日期使用"X月X日"格式

2. **历史记录卡片**:
   - 保持现有信息:人格名称、时间、时长、字数
   - 右上角操作区:复制按钮、删除按钮
   - 删除按钮使用 Trash2Icon,视觉上与复制按钮区分

3. **删除确认**:
   - 直接删除,不弹窗确认
   - 通过 toast 提示删除成功
   - 考虑到历史记录可以重新生成,简化操作流程

## 涉及文件

- `src/main.tsx`: 前端首页重构,移除内联 JSX,使用组件
- `src/components/home/QuickStartCard.tsx`: 快速开始卡片组件
- `src/components/home/VoiceInputStatsCard.tsx`: 语音输入成效卡片组件
- `src/components/home/index.ts`: 组件导出文件
- `src-tauri/src/data.rs`: 新增删除历史记录命令
- `src-tauri/src/lib.rs`: 注册新命令

## 测试与验证

### 编译验证

1. **TypeScript 类型检查**:
   ```bash
   pnpm exec tsc --noEmit
   ```
   结果: ✅ 通过,无类型错误

2. **Rust 编译检查**:
   ```bash
   cargo check --manifest-path=src-tauri/Cargo.toml
   ```
   结果: ✅ 通过,编译成功

3. **前端构建**:
   ```bash
   pnpm build
   ```
   结果: ✅ 通过,构建成功
   - 输出: dist/index.html (0.40 kB)
   - 输出: dist/assets/index-DnobFJqu.css (36.69 kB)
   - 输出: dist/assets/index-CTI8Gf0m.js (409.10 kB)

### 功能验证(手动测试)

需要在 `pnpm tauri dev` 启动后手动测试:

1. **时间分段展示**:
   - [ ] 今天的记录显示在"今天"分组
   - [ ] 昨天的记录显示在"昨天"分组
   - [ ] 更早的记录按日期分组(如"5月23日")

2. **删除功能**:
   - [ ] 点击删除按钮,记录被删除
   - [ ] 删除后,统计数据自动更新
   - [ ] 删除后,toast 提示成功

3. **复制功能**:
   - [ ] 点击复制按钮,文本复制到剪贴板
   - [ ] toast 提示复制成功

4. **边界情况**:
   - [ ] 没有历史记录时,显示空状态提示
   - [ ] 只有一条记录时,删除后显示空状态
   - [ ] 删除失败时,显示错误提示

5. **组件拆分验证**:
   - [ ] `QuickStartCard` 当前处于隐藏状态，如需验证需要先恢复渲染或提供替代入口
   - [ ] 语音输入成效卡片功能正常
   - [ ] 组件间无状态泄漏

## 执行复盘

### 遇到的问题

1. **组件拆分需求**:
   - **问题发现**: 用户指出 main.tsx 代码全部集中,超过 2000 行,可维护性差
   - **问题原因**: T024 初始实现只关注功能,未考虑代码组织
   - **解决方案**: 
     - 创建 `src/components/home/` 目录
     - 将"快速开始"卡片抽取为 `QuickStartCard` 组件
     - 将"语音输入成效"卡片抽取为 `VoiceInputStatsCard` 组件
     - 通过 props 传递状态和回调函数
   - **验证方法**: TypeScript 类型检查和前端构建均通过

### 解决流程

1. **问题发现**: 完成 T024 功能实现后,用户提出"怎么不抽组件?代码现在全集中在 main 中了"
2. **初步诊断**: 确认 main.tsx 确实过大,首页两个卡片各有 100+ 行 JSX
3. **尝试方案**: 
   - 创建组件目录结构
   - 定义清晰的 props 接口
   - 将 JSX 和相关类型定义移到组件文件
4. **最终方案**: 
   - 创建两个独立组件文件
   - 在 main.tsx 中通过 props 传递所有必要的状态和回调
   - 使用 index.ts 统一导出
5. **验证结果**: 编译检查和构建均通过,代码量显著减少

### 经验总结

- **及时重构**: 功能实现后应立即考虑代码组织,避免技术债累积
- **组件拆分原则**: 单个文件超过 500 行或单个组件超过 200 行 JSX 时应考虑拆分
- **Props 设计**: 组件 props 应包含所有必要的状态和回调,避免隐式依赖
- **类型定义**: 组件内部使用的类型应在组件文件中定义或导入,保持自包含

## 未完成事项

- 首页历史记录、统计和删除能力已实现。
- `QuickStartCard` 当前被隐藏，首页没有可见语音输入入口。
- 快捷键录音完成事件尚未接入前端处理流程。

## 后续建议

1. **继续组件拆分**: 
   - T025-T027 在实现人格页、热词页、设置页时,应直接创建独立组件
   - 避免在 main.tsx 中继续堆积代码

2. **考虑状态管理**: 
   - 当前通过 props 传递状态和回调,层级较深时可能出现 prop drilling
   - 后续可考虑引入 Context 或状态管理库

3. **下一步任务**: 
   - 继续执行 T025: 人格页整合
   - 将人格管理卡片移动到人格页,并创建独立组件
