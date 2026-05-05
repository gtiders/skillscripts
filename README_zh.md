# skillscripts

极速脚本搜索与技能检索 CLI。

## 它是什么

`skillscripts`（别名 `sks`）是一个本地优先的命令行工具，提供两种核心能力：

**脚本极速搜索**：
- 你有大量脚本散落在各处，难以找到
- 你想快速定位某个功能的脚本
- 你需要一个轻量的脚本管理器

**技能极速检索**：
- 你是 AI Agent 开发者，需要管理技能库
- 你想快速检索可复用的技能
- 你需要为 Agent 提供工具调用能力

## 核心特性

### 双模式搜索

同一套命令服务于两种场景：

**脚本搜索模式**：
- 按功能快速定位脚本文件
- 轻量级脚本管理，无需复杂数据库
- 模糊匹配即时返回结果

**技能检索模式**：
- 为 AI Agent 检索可复用技能
- 输出 YAML 格式，可直接用于工具调用
- 管理 Agent 开发的技能库

两种模式共享相同的输出格式（YAML，包含 name、description、tags、path），便于将脚本作为 Agent 工具使用。

### 即时扫描

- 并行文件扫描，毫秒级响应
- 进程内内存缓存，提升重复操作性能
- 自动检测文件编码，跳过二进制文件
- 支持 gitignore 规则

### 智能匹配

- 对 `name`、`description`、`tags` 进行模糊匹配
- 智能评分排序
- 路径不参与搜索，减少噪音

### YAML 头部

任何脚本添加 YAML 头部即可被索引：

```python
# ---
# name: resize_image
# description: 使用 PIL 调整图片尺寸
# tags: [image, python]
# ---
from PIL import Image
```

这个头部具有双重作用：
- **对于脚本搜索**：提供元数据便于快速识别
- **对于技能检索**：定义工具接口供 Agent 调用

支持的注释风格：`#`、`//`、`--`、`%`、`/**` 等。

### 交互式选择

基于 skim 的 TUI 界面，支持实时预览，适用于脚本选择和技能浏览。

## 安装

从 release 安装：
- https://github.com/gtiders/skillscripts/releases/latest

从源码安装：

```bash
cargo install --path .
```

## 快速上手

```bash
# 初始化配置
sks init

# 搜索脚本/技能
sks search image

# 列出所有脚本/技能
sks list

# 交互式选择
sks pick
```

## 命令

| 命令 | 说明 |
|------|------|
| `init` | 创建配置文件。`--local` 创建项目级配置。 |
| `config` | 查看当前配置。 |
| `search <query>` | 模糊搜索，输出 YAML。 |
| `list` | 列出所有脚本/技能，输出 YAML。 |
| `pick` | 交互式 TUI 选择器。 |

## 输出格式

`search` 和 `list` 输出 YAML：

```yaml
- name: resize_image
  tags:
    - image
    - python
  description: 使用 PIL 调整图片尺寸
  path: ./scripts/resize_image.py
```

## 配置

配置文件位置：
- 全局：`~/.config/skillscripts/skillscripts.yaml`
- 本地：`./skillscripts.yaml`（项目级，与全局合并）

### 配置示例

```yaml
scan_paths:
  - skills
  - ./scripts
  - ~/projects/utils
ignore_patterns:
  - target
  - .git
  - node_modules
max_file_size: 1MB
search_limit: 10
report_parse_errors: true
```

### 配置项说明

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `scan_paths` | 扫描路径列表 | `["."]` |
| `ignore_patterns` | 忽略模式 | `[]` |
| `max_file_size` | 最大文件大小 | `1MB` |
| `search_limit` | 搜索结果数量限制 | `5` |
| `report_parse_errors` | 报告解析错误 | `false` |
| `copy_to_clipboard_on_pick` | pick 后复制路径到剪贴板 | `false` |

## YAML 头部规范

### 必填字段

| 字段 | 说明 |
|------|------|
| `name` | 脚本/技能名称 |
| `description` | 脚本/技能描述 |

### 可选字段

| 字段 | 说明 |
|------|------|
| `tags` | 标签列表 |
| `args` | 参数定义 |
| `version` | 版本号 |
| `command_template` | 命令模板 |

### 示例

**Python**：
```python
# ---
# name: disk_check
# description: 检查磁盘使用情况
# tags: [ops, monitoring]
# args:
#   path:
#     type: string
#     description: 目标路径
#     required: false
# ---
import shutil
shutil.disk_usage(path)
```

**Shell**：
```bash
#!/bin/bash
# ---
# name: git_log
# description: 显示最近提交
# tags: [git, vcs]
# ---
git log --oneline -10
```

**JavaScript**：
```javascript
// ---
// name: fetch_data
// description: 获取远程数据
// tags: [http, async]
// ---
const response = await fetch(url);
```

## License

MIT
