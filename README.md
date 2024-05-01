此工具用于实现 KDE Wayland 下的触控板手势，相比 `touchegg` 或 `libinput-gestures`，支持了同一手势在不同窗口下执行不同操作。
其参考了 https://github.com/xremap/xremap 。

首先安装 [kwin-without-gestures](https://github.com/chiyuki0325/arch-packages/raw/main/kwin-without-gestures/0001-kwin-disable-gestures.diff) 补丁或[其 arch 软件包](https://github.com/chiyuki0325/arch-packages/tree/main/kwin-without-gestures)，然后编译安装本软件。  
编译时不要使用 `--release` 参数。

之后安装对应的 KWin 脚本，并且配置好 `touchegg` 或 `libinput-gestures` 进行手势识别，运行 `qdbus` 或 `dbus-send` 命令即可调用本软件。

```bash
qdbus ink.chyk.GesturesHelper /ink/chyk/GesturesHelper ink.chyk.GesturesHelper.InvokeGesture 0
```

由于是自用，所以并没有做读取配置文件，而是写死在 `main.rs` 中，如果需要使用，请自行修改。
