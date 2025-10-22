# GXRTools 编译和部署指南

> **版本**: 1.0  
> **最后更新**: 2024年  
> **状态**: ✅ 已验证

---

## 📋 目录

- [系统要求](#系统要求)
- [编译环境准备](#编译环境准备)
- [编译步骤](#编译步骤)
- [编译验证](#编译验证)
- [部署指南](#部署指南)
- [常见问题](#常见问题)
- [性能优化](#性能优化)

---

## 🖥️ 系统要求

### 操作系统支持

| 操作系统 | 架构 | 状态 |
|---------|------|------|
| Linux | x86_64 | ✅ 完全支持 |
| Linux | aarch64 | ✅ 支持 |
| Windows | x86_64 | ✅ 支持 |
| macOS | x86_64/arm64 | ⚠️ 未测试 |

### 软件依赖

- **Rust**: 1.70.0 或更高版本
- **Cargo**: 随 Rust 一起安装
- **gcc/clang**: Linux 编译需要
- **Oracle Instant Client**: Oracle 模块需要（可选）

---

## 🔧 编译环境准备

### 1. 安装 Rust

#### Linux/macOS
```bash
# 安装 rustup（Rust 安装工具）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 配置环境变量
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### Windows
1. 下载并运行 [rustup-init.exe](https://rustup.rs/)
2. 按照提示完成安装
3. 重启命令提示符/PowerShell
4. 验证安装：`rustc --version`

### 2. 配置 Rust 工具链

```bash
# 安装稳定版工具链（推荐）
rustup install stable
rustup default stable

# 更新工具链
rustup update

# 添加目标平台（跨平台编译）
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
```

### 3. 配置国内镜像（可选，加速下载）

创建或编辑 `~/.cargo/config.toml`：

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"

# 或使用清华镜像
[source.tuna]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"
```

### 4. 安装额外依赖（Linux）

#### Debian/Ubuntu
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev
```

#### CentOS/RHEL
```bash
sudo yum install -y \
    gcc \
    gcc-c++ \
    make \
    openssl-devel \
    sqlite-devel
```

#### Arch Linux
```bash
sudo pacman -S base-devel openssl sqlite
```

---

## 🔨 编译步骤

### 基础编译

#### 1. 克隆或进入项目目录
```bash
cd /path/to/gxrtools
```

#### 2. 检查依赖
```bash
# 验证 Cargo.toml
cat Cargo.toml

# 下载依赖（不编译）
cargo fetch
```

#### 3. 开发版本编译
```bash
# 编译（包含调试信息，编译快但运行慢）
cargo build

# 编译后的二进制文件位置
ls -lh target/debug/gxtools
```

#### 4. 发布版本编译（推荐）
```bash
# 优化编译（编译慢但运行快）
cargo build --release

# 编译后的二进制文件位置
ls -lh target/release/gxtools

# 查看文件大小（通常 15-30MB）
du -h target/release/gxtools
```

### 特定平台编译

#### Linux 编译
```bash
# 本地平台
cargo build --release

# 指定目标平台
cargo build --target x86_64-unknown-linux-gnu --release

# 静态链接（便于部署）
RUSTFLAGS='-C target-feature=+crt-static' \
    cargo build --target x86_64-unknown-linux-gnu --release
```

#### Windows 编译（在 Linux 上交叉编译）
```bash
# 安装交叉编译工具
sudo apt-get install mingw-w64
rustup target add x86_64-pc-windows-gnu

# 编译 Windows 版本
cargo build --target x86_64-pc-windows-gnu --release
```

#### Windows 编译（在 Windows 上）
```powershell
# PowerShell
cargo build --release

# 输出位置
dir target\release\gxtools.exe
```

### 编译选项优化

#### 最小化体积
在 `Cargo.toml` 中添加：

```toml
[profile.release]
opt-level = "z"     # 优化体积
lto = true          # 链接时优化
codegen-units = 1   # 减少代码单元
strip = true        # 移除符号信息
```

#### 最大化性能
```toml
[profile.release]
opt-level = 3       # 最高优化级别
lto = "fat"         # 完全 LTO
codegen-units = 1
panic = "abort"     # 减少 panic 处理代码
```

---

## ✅ 编译验证

### 1. 编译检查
```bash
# 快速语法检查（不生成二进制）
cargo check

# 检查所有目标（包括测试）
cargo check --all-targets

# 针对特定平台检查
cargo check --target x86_64-unknown-linux-gnu
```

### 2. 代码质量检查
```bash
# Clippy 静态分析
cargo clippy

# 严格模式（将警告视为错误）
cargo clippy -- -D warnings

# 格式化检查
cargo fmt -- --check

# 自动格式化
cargo fmt
```

### 3. 运行测试
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test utils::tests

# 显示测试输出
cargo test -- --nocapture

# 单线程运行测试
cargo test -- --test-threads=1
```

### 4. 功能验证
```bash
# 查看帮助信息
./target/release/gxtools --help

# 查看版本
./target/release/gxtools --version

# 测试 Ping 功能
./target/release/gxtools net ping -t 127.0.0.1

# 测试端口扫描
./target/release/gxtools pentest scan -t 127.0.0.1 -p 80,443
```

### 5. 性能测试
```bash
# 安装 hyperfine（性能测试工具）
cargo install hyperfine

# 测试 Ping 性能
hyperfine './target/release/gxtools net ping -t 192.168.1.1-10'

# 测试端口扫描性能
hyperfine './target/release/gxtools pentest scan -t 127.0.0.1 -p 1-1000'
```

---

## 📦 部署指南

### 单文件部署

#### Linux
```bash
# 复制二进制文件
sudo cp target/release/gxtools /usr/local/bin/

# 设置执行权限
sudo chmod +x /usr/local/bin/gxtools

# 验证安装
gxtools --version

# 创建配置目录
mkdir -p ~/.config/gxtools
```

#### Windows
```powershell
# 复制到 Program Files
Copy-Item target\release\gxtools.exe "C:\Program Files\gxtools\"

# 添加到 PATH（永久）
$env:Path += ";C:\Program Files\gxtools"
[Environment]::SetEnvironmentVariable("Path", $env:Path, "Machine")

# 验证
gxtools --version
```

### 完整部署（包含配置文件）

#### 部署结构
```
/opt/gxtools/
├── gxtools                 # 主程序
├── cmd.yaml                # 命令配置
├── fingerprints.yaml       # 指纹配置
├── instantclient/          # Oracle 客户端（可选）
│   ├── libclntsh.so
│   └── ...
└── output/                 # 输出目录
    ├── ping/
    ├── portscan/
    ├── ssh/
    └── ...
```

#### 部署脚本（Linux）
```bash
#!/bin/bash
# deploy.sh

# 设置变量
INSTALL_DIR="/opt/gxtools"
BIN_NAME="gxtools"

# 创建目录
sudo mkdir -p "$INSTALL_DIR"
sudo mkdir -p "$INSTALL_DIR/output"

# 复制文件
sudo cp target/release/gxtools "$INSTALL_DIR/"
sudo cp cmd.yaml "$INSTALL_DIR/" 2>/dev/null || true
sudo cp fingerprints.yaml "$INSTALL_DIR/" 2>/dev/null || true

# 复制 Oracle 客户端（如果存在）
if [ -d "instantclient" ]; then
    sudo cp -r instantclient "$INSTALL_DIR/"
fi

# 设置权限
sudo chmod +x "$INSTALL_DIR/$BIN_NAME"
sudo chmod 755 "$INSTALL_DIR"

# 创建符号链接
sudo ln -sf "$INSTALL_DIR/$BIN_NAME" /usr/local/bin/$BIN_NAME

# 创建配置文件模板
cat <<EOF | sudo tee "$INSTALL_DIR/cmd.yaml" > /dev/null
linux_commands:
  - whoami
  - hostname
  - uname -a

mysql_commands:
  - SELECT VERSION()
  - SHOW VARIABLES

oracle_commands:
  - SELECT * FROM v\$version
EOF

echo "✅ 部署完成！"
echo "📍 安装位置: $INSTALL_DIR"
echo "🔗 可执行文件: /usr/local/bin/$BIN_NAME"
echo "⚙️  配置文件: $INSTALL_DIR/cmd.yaml"
```

#### 部署脚本（Windows PowerShell）
```powershell
# deploy.ps1
$InstallDir = "C:\Program Files\gxtools"

# 创建目录
New-Item -ItemType Directory -Force -Path $InstallDir
New-Item -ItemType Directory -Force -Path "$InstallDir\output"

# 复制文件
Copy-Item "target\release\gxtools.exe" $InstallDir
Copy-Item "cmd.yaml" $InstallDir -ErrorAction SilentlyContinue
Copy-Item "fingerprints.yaml" $InstallDir -ErrorAction SilentlyContinue

# 添加到 PATH
$Path = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($Path -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$Path;$InstallDir",
        "Machine"
    )
}

Write-Host "✅ 部署完成！" -ForegroundColor Green
Write-Host "📍 安装位置: $InstallDir"
Write-Host "⚙️  请重新打开终端以使 PATH 生效"
```

### Docker 部署

#### Dockerfile
```dockerfile
# 多阶段构建
FROM rust:1.75-slim as builder

WORKDIR /app

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制项目文件
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build.rs ./

# 编译（发布版本）
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建用户
RUN useradd -m -u 1000 gxtools

# 复制二进制文件
COPY --from=builder /app/target/release/gxtools /usr/local/bin/

# 创建输出目录
RUN mkdir -p /opt/gxtools/output && \
    chown -R gxtools:gxtools /opt/gxtools

USER gxtools
WORKDIR /opt/gxtools

# 默认命令
CMD ["gxtools", "--help"]
```

#### 构建和运行
```bash
# 构建镜像
docker build -t gxtools:latest .

# 运行容器
docker run --rm gxtools:latest gxtools --version

# 带挂载目录运行
docker run --rm \
    -v $(pwd)/output:/opt/gxtools/output \
    gxtools:latest \
    gxtools net ping -t 8.8.8.8
```

---

## ❓ 常见问题

### 编译问题

#### Q1: 编译时内存不足
```bash
# 限制并行编译任务数
cargo build --release -j 2

# 或设置环境变量
export CARGO_BUILD_JOBS=2
cargo build --release
```

#### Q2: 依赖下载失败
```bash
# 清理缓存
cargo clean

# 删除锁文件重新生成
rm Cargo.lock
cargo build

# 或使用离线模式（需提前下载依赖）
cargo build --offline
```

#### Q3: OpenSSL 链接错误（Linux）
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# CentOS/RHEL
sudo yum install openssl-devel

# 或使用 vendored 特性（静态链接）
cargo build --release --features vendored
```

#### Q4: Windows 上编译错误
```powershell
# 安装 Visual Studio Build Tools
# 下载: https://visualstudio.microsoft.com/downloads/

# 或使用 GNU 工具链
rustup default stable-x86_64-pc-windows-gnu
```

### 运行问题

#### Q5: 找不到动态库
```bash
# Linux - 设置 LD_LIBRARY_PATH
export LD_LIBRARY_PATH=/opt/gxtools/instantclient:$LD_LIBRARY_PATH

# 或永久添加
echo 'export LD_LIBRARY_PATH=/opt/gxtools/instantclient:$LD_LIBRARY_PATH' >> ~/.bashrc
```

#### Q6: 权限不足
```bash
# Linux - 某些功能需要 root 权限
sudo ./gxtools pentest scan -t 192.168.1.0/24

# 或使用 capabilities（更安全）
sudo setcap cap_net_raw+ep ./gxtools
```

#### Q7: 配置文件找不到
```bash
# 在当前目录创建配置文件
cp /opt/gxtools/cmd.yaml .

# 或使用绝对路径
./gxtools check linux --yaml /opt/gxtools/cmd.yaml
```

---

## ⚡ 性能优化

### 编译时优化

#### 使用 LTO（链接时优化）
```toml
[profile.release]
lto = "fat"
codegen-units = 1
```

**效果**: 体积减小 10-20%，性能提升 5-10%

#### 使用 CPU 特定优化
```bash
# 针对当前 CPU 优化
RUSTFLAGS="-C target-cpu=native" cargo build --release

# 针对特定 CPU 特性
RUSTFLAGS="-C target-feature=+avx2,+sse4.2" cargo build --release
```

**注意**: 编译的二进制可能无法在其他 CPU 上运行

#### 并行编译加速
```bash
# 使用所有 CPU 核心
cargo build --release -j $(nproc)

# 或在 .cargo/config.toml 中设置
[build]
jobs = 8
```

### 运行时优化

#### 调整并发数
```bash
# Ping 扫描（推荐 100-200）
gxtools net ping -t 192.168.1.0/24 -c 200

# 端口扫描（推荐 200-500）
gxtools pentest scan -t 192.168.1.0/24 -c 500
```

#### 使用存活探测
```bash
# 先探测存活主机，减少无效扫描
gxtools pentest scan -t 192.168.1.0/24 --live
```

---

## 📊 编译结果参考

### 二进制文件大小

| 配置 | Linux | Windows | 说明 |
|------|-------|---------|------|
| Debug | ~180MB | ~200MB | 包含调试信息 |
| Release | ~25MB | ~30MB | 基础优化 |
| Release + LTO | ~20MB | ~25MB | 链接时优化 |
| Release + strip | ~15MB | ~20MB | 移除符号表 |

### 编译时间参考

| 步骤 | 时间（首次） | 时间（增量） |
|------|-------------|-------------|
| cargo check | ~2分钟 | ~5秒 |
| cargo build | ~5分钟 | ~30秒 |
| cargo build --release | ~8分钟 | ~1分钟 |

**注**: 时间取决于 CPU 性能和依赖缓存状态

---

## 🔗 相关资源

### 官方文档
- [Rust 官方文档](https://doc.rust-lang.org/)
- [Cargo 手册](https://doc.rust-lang.org/cargo/)
- [Rustup 文档](https://rust-lang.github.io/rustup/)

### 工具
- [rust-analyzer](https://rust-analyzer.github.io/) - IDE 支持
- [cargo-watch](https://github.com/watchexec/cargo-watch) - 自动编译
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat) - 体积分析

### 优化指南
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Cargo 配置文档](https://doc.rust-lang.org/cargo/reference/config.html)

---

## 📝 检查清单

编译前检查：
- [ ] Rust 版本 >= 1.70.0
- [ ] 磁盘空间 >= 5GB
- [ ] 系统依赖已安装
- [ ] 网络连接正常（首次编译）

编译后检查：
- [ ] 编译无错误无警告
- [ ] 二进制文件存在且可执行
- [ ] `--help` 命令正常
- [ ] `--version` 显示正确
- [ ] 基础功能测试通过

部署前检查：
- [ ] 目标系统兼容性
- [ ] 配置文件准备完毕
- [ ] 输出目录权限正确
- [ ] 依赖库已安装

---

**最后更新**: 2024年  
**维护者**: GX Tools Team  
**许可证**: 根据项目实际情况