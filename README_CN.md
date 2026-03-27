<img height="200" alt="image" src="https://github.com/user-attachments/assets/c324645d-683c-4cc6-b744-60d8b6bbfc58" />

[`English`](README.md) | **中文**

# Capsense

在 Windows 上，将 `CapsLock` 键用于切换输入法。

## 为什么？

在 Windows 上，用于切换输入语言的默认快捷键（例如 `Shift` 和 `Win + Space`）
可能会让人对当前输入法状态感到困惑。

macOS 通过将常被闲置的 `CapsLock` 键用作输入源切换键以解决这个问题。**Capsense** 将此行为带到 Windows，
通过轻按 `CapsLock` 键触发 IME 切换，同时通过长按保留 `CapsLock` 的原始用途。

## 功能

- 短按 `CapsLock` 来切换输入法。
- 长按以切换大写锁定。
- 在后台高效运行，占用资源极少。
- 可轻松更改轻按判定阈值和所触发的操作。

## 使用方法

直接运行可执行文件，即可开始监控 `CapsLock` 事件。

在已有实例运行的情况下，再次从文件资源管理器运行可执行文件，将出现 **实例管理器** 窗口，
可用于停止实例、重新加载配置或编辑 `config.toml` 文件。

默认情况下，从文件资源管理器启动时，Capsense 将使用弹窗显示必要的提示信息，
通过其他方式（例如命令行）启动时则不会弹窗。你可以通过命令行参数来修改这一行为。

建议在输入法设置中禁用 `Shift` 键切换 IME 状态的功能，以获得最佳体验，因为这可能会让你困惑。
使用 `CapsLock` 切换键盘布局，使用 `Win+Space` 切换当前键盘布局的主输入法。

### CLI

程序支持以下命令行参数：

- `-d, --daemon`：在后台启动 Capsense。
- `-s, --stop`：停止正在运行的 Capsense 实例。
- `-r, --reload`：让正在运行的实例从 `config.toml` 重新加载配置。
- `-S, --status`：检查是否有 Capsense 实例在运行并显示其 PID。
- `--gui`：允许 Capsense 显示 GUI 窗口。
- `--headless`：阻止 Capsense 显示任何 GUI 窗口。
- `--startup <enable|disable>`：启用或禁用开机自启。

## 配置

首次运行时，Capsense 会在同一目录下创建 `config.toml` 文件。你可以自定义以下内容：

- `tap_threshold_ms`：超过这个时间的按压将被视为长按。默认为 `300` ms。
- `tap_action`：在轻按时执行的动作。支持的动作为：
    - `shortcut`：触发一个键盘快捷键（由 `tap_shortcut` 定义）。
    - `switch_layout`：（默认）轮换输入布局。
- `tap_shortcut`：要触发的快捷键（默认为 `["LWIN", "SPACE"]`）。支持的按键有：
    - `LWIN`（或 `WIN`)
    - `SPACE`
    - `LCONTROL`（或 `CTRL`)
    - `LSHIFT`（或 `SHIFT`)
    - `LMENU`（或 `ALT`)
    - `CAPSLOCK`
- `layouts`：当 `tap_action` 设置为 `switch_layout` 时要轮换的一组输入布局 ID。
    - 默认：`[0x0804, 0x0409]`（`zh-CN` 和 `en-GB`）。
    - 有关更多布局 ID，请参见[微软文档](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-lcid/70feba9f-294e-491e-b6eb-56532684c37f)。其他常见的有：
        - `0x0404`：繁体中文
        - `0x0411`：日文
        - `0x0412`：韩文
- `no_en`：若启用，Capsense 将防止中文输入法在布局切换或焦点更改后进入英文模式。（默认为 `true`）。

## 许可

```
Capsense 是自由软件；
您可以根据自由软件基金会发布的 GNU 通用公共许可证第 3 版，
或（由您自行选择）任何更高版本的条款对其进行再发布和/或修改。

我们发布 Capsense 的目的是希望它能够发挥作用，但不提供任何担保；
甚至不包含适销性或适用于特定用途的默示担保。有关详细信息，请参阅
GNU 通用公共许可证。

您应该已经随本程序一起收到了 GNU 通用公共许可证的副本。
如果没有，请参见 <https://www.gnu.org/licenses/>。
```