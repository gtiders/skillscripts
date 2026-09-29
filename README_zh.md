# sks

基于注册表的本地脚本启动器。`sks` 读取 YAML 注册项，提供脚本列表、检索、执行，通过命令行搜索。

## 核心功能

- 使用 Python 风格 ASCII 名称精确执行脚本：`[A-Za-z_][A-Za-z0-9_]*`。
- 支持 YAML 注册表、配置导入、描述和标签。
- 交互式 Picker，支持源码预览和语法高亮。
- 通过命令行搜索脚本，以 YAML 输出元数据和源码路径。
- 执行前将脚本复制到当前目录的 `.sks/<文件名>`。
- 从 GitHub 最新 Release 自更新，按编译目标选择资源并校验文件摘要。
- 提供脚本使用和脚本创建的内置 Skill 指引。

## 环境依赖

- 从源码构建需要 Rust 1.85 或更高版本。
- 每个注册脚本所需的运行时，例如 Python，由脚本自行决定。

## 安装部署

从源码构建并安装：

```bash
git clone https://github.com/gtiders/skillscripts.git
cd skillscripts
cargo install --path .
```

初始化全局配置：

```bash
sks init
```

配置目录为 `~/.config/sks`。`init` 会创建 `sks.yaml`、空的 `scripts.yaml`，并将 `sks-script-use` 和 `sks-script-create` Agent Skill 安装到 `~/.agents/skills`。

## 使用方法

```bash
sks list
sks search "markdown pdf"
sks pick
sks run <name> [args...]
sks skill use
sks skill create
sks update
sks update --check
sks update --force
```

`run` 按 `name` 精确匹配。名称后的所有参数都会追加到注册命令。执行前，脚本源文件会复制到当前目录的 `.sks/<文件名>`；同名文件直接覆盖。即使命令执行失败，复制也已经完成。

`list` 以 YAML 输出完整注册表。`pick` 用卡片展示每个脚本的名称、路径、命令、说明和标签，右侧预览高亮源码。搜索采用模糊匹配，并高亮命中的字符。上下键或鼠标点击可选中卡片；从最后一张向下或从第一张向上会循环到另一端，卡片区忽略鼠标滚轮。Enter 输出所选注册项，Esc 取消。超出终端高度的卡片可用 Alt+上/下滚动。源码按预览宽度折行；鼠标停在预览区时可用滚轮滚动，也可用 Ctrl+D 和 Ctrl+B。卡片内容按当前搜索词、主题和宽度缓存；预览的加载和折行在后台完成。

卡片和预览使用同一套 [Chromata](https://docs.rs/chromata/) Base24 主题颜色。默认主题是 `Catppuccin Frappe`。用 `sks themes` 列出所有主题名称；只通过 `~/.config/sks/sks.yaml` 中的 `picker.theme` 更改主题。例如：

```yaml
picker:
  theme: Catppuccin Frappe
```

`update` 请求 GitHub 最新 Release，按二进制编译时的 Rust target（包括 GNU 或 musl）选择资源，校验 `checksums.txt` 后替换当前可执行文件。`--check` 只检查不安装；`--force` 在版本比较无法确认时仍执行安装。

### 搜索

运行 `sks search "<查询词>"`，按相关性排序并输出 YAML。每条结果包含注册名称、解析后的源码路径、命令，以及可选的说明和标签。搜索只读；执行时使用精确名称 `sks run <name> [args...]`。`--limit N` 可修改默认的 5 条结果，`--tag TAG` 可重复指定排序提示。没有结果时输出 `[]`。

## 配置说明

全局配置文件：`~/.config/sks/sks.yaml`

```yaml
imports:
  - scripts.yaml
  - imports/tools.yaml

scripts: []
```

脚本注册示例：

```yaml
scripts:
  - name: ase_to_xyz
    path: tools/ase2xyz.py
    command: python {{path}}
    comment: 将 ASE 可读取的结构文件转换为 extended XYZ
    tags: [ase, structure, extxyz, conversion]
```

规则：

- `name` 必填、大小写敏感，且全局唯一。
- 名称必须匹配 `[A-Za-z_][A-Za-z0-9_]*`。空字符串、Unicode 字符、数字开头、点号、连字符、斜杠和空格均非法。
- `path` 必须是相对 Unix 风格路径，相对于定义它的 YAML 文件解析。
- 只有全局配置可以声明 `imports`；被导入文件不能继续导入。
- `command` 必须包含 `{{path}}`，运行时替换为解析后的脚本路径。
- `comment` 和 `tags` 可选。标签用于搜索排序加权。

## 常见问题

### `Global config not found`

先运行 `sks init`，再向 `~/.config/sks/sks.yaml` 或导入的 YAML 文件添加注册项。

### `invalid script name`

将名称改为 ASCII Python 风格标识符，例如 `convert_csv` 或 `_internal`。

### `unknown script name`

运行 `sks list` 查看已注册名称，并使用完全一致的名称。匹配区分大小写，不进行模糊猜测。

### `command` 校验失败

在命令中加入 `{{path}}`，例如 `python {{path}}`。

### `sks update` 找不到资源

Release 必须提供与当前二进制编译目标对应的压缩包以及匹配的 `checksums.txt`。检查网络连接和 GitHub 最新 Release 的资源列表。
