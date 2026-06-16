# sks

一个极简的、基于注册表的脚本检索与执行 CLI。

## 概览

`sks` 只读取一个全局配置文件：`~/.config/sks/sks.yaml`。  
它不会扫描目录，也不会解析脚本头部元数据。

每个脚本注册项只保留三个字段：

- `id`
- `path`
- `command`

`path` 相对于“定义它的 YAML 文件”解析。只有全局配置允许声明 `imports`。

## 配置格式

全局配置：

```yaml
imports:
  - lang/python.yaml

scripts:
  - id: 1
    path: scripts/hello.py
    command: python {{path}}
```

被导入配置：

```yaml
scripts:
  - id: 2
    path: tools/build.py
    command: python {{path}}
```

规则：

- 只允许相对路径
- imported 文件不能再声明 `imports`
- `id` 必须全局唯一
- `command` 必须包含 `{{path}}`

## 命令

```bash
sks init
sks list
sks pick
sks run 1 foo --bar baz
```

- `init` 创建 `~/.config/sks/sks.yaml`
- `list` 以 YAML 输出所有已注册脚本
- `pick` 打开交互式选择器，并显示表格化列表与语法高亮预览
- `run <id> [args...]` 替换 `command` 中的 `{{path}}`，并把剩余参数全部追加到命令尾部

## Picker

`pick` 的列表包含三列：

- `ID`
- `PATH`
- `COMMAND`

右侧预览区会直接渲染完整脚本文件内容，并使用内嵌 `syntect` 做语法高亮。当前默认主题是 GitHub Dark，预览背景由 skim 控制。

## run 语义

`run` 的设计是刻意保持极简：

```bash
sks run 12 input.txt --mode fast
```

它的行为是：

1. 找到 `id: 12`
2. 替换 `command` 中的 `{{path}}`
3. 把 `input.txt --mode fast` 原样追加到命令后面

也就是说，`run` 在 `<id>` 之后不再保留自己的选项解析层。

## 安装

从源码安装：

```bash
cargo install --path .
```
