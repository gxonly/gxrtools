# 弱口令扫描模块使用指南

## 📋 概述

弱口令扫描模块是 GXRTools 提供的安全测试工具，用于检测目标系统是否存在弱口令漏洞。支持多种服务的弱口令检测，包括 SSH、RDP、Tomcat、Nacos 等。

⚠️ **重要提示**: 本工具仅用于授权的安全测试，请勿用于未经授权的系统。

---

## 🎯 支持的服务

### 1. SSH (端口 22)
- 支持密码认证
- 实时连接测试
- 支持自定义用户名和密码字典

### 2. RDP (端口 3389)
- 端口连通性检测
- 预留完整认证接口

### 3. Tomcat Manager
- `/manager/html` - Web 管理界面
- `/manager/text` - 文本管理界面
- `/host-manager/html` - 主机管理界面
- 端口: 8080, 8009 (AJP)

### 4. Nacos
- `/nacos/v1/auth/login` - 登录接口
- 端口: 8848
- 支持 v1 版本 API

### 5. 通用 Web 应用
- 支持常见登录路径探测
- HTTP Basic Auth 认证
- 表单 POST 登录
- 端口: 80, 443, 8080

---

## 🚀 快速开始

### 基础用法

```bash
# 扫描 SSH 服务
gxtools pentest weakpass -t 192.168.1.1 -s ssh

# 扫描 Tomcat 服务
gxtools pentest weakpass -t 192.168.1.1 -s tomcat

# 扫描所有支持的服务
gxtools pentest weakpass -t 192.168.1.1 -s all
```

### 扫描网段

```bash
# 扫描整个 C 段
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh

# 扫描 IP 范围
gxtools pentest weakpass -t 192.168.1.1-50 -s all

# 扫描多个 IP
gxtools pentest weakpass -t 192.168.1.1,192.168.1.5,192.168.1.10 -s ssh
```

---

## 📚 命令参数

### 必需参数

| 参数 | 短选项 | 说明 | 示例 |
|------|--------|------|------|
| `--targets` | `-t` | 目标 IP 或 IP 段 | `-t 192.168.1.0/24` |

### 可选参数

| 参数 | 短选项 | 默认值 | 说明 |
|------|--------|--------|------|
| `--service` | `-s` | `all` | 服务类型 (ssh/rdp/tomcat/nacos/web/all) |
| `--usernames` | `-u` | `usernames.txt` | 用户名（字符串或.txt字典文件） |
| `--passwords` | `-p` | `passwords.txt` | 密码（字符串或.txt字典文件） |
| `--concurrency` | `-c` | `10` | 最大并发数 |
| `--timeout` | `-T` | `5` | 连接超时时间（秒）|
| `--output` | `-o` | `false` | 是否输出到 Excel |

### 用户名和密码参数说明

**智能识别规则**:
- 如果参数以 `.txt` 结尾 → 作为字典文件加载
- 否则 → 作为字符串处理（支持逗号分隔多个值）

**示例**:
```bash
# 使用字典文件
-u usernames.txt -p passwords.txt

# 使用单个字符串
-u admin -p admin123

# 使用多个字符串（逗号分隔）
-u admin,root,test -p admin123,password,123456

# 混合使用
-u admin,root -p passwords.txt
```

---

## 📖 详细使用示例

### 1. SSH 弱口令扫描

```bash
# 基础扫描
gxtools pentest weakpass -t 192.168.1.1 -s ssh

# 使用自定义字典文件
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s ssh \
    -u my_users.txt \
    -p my_pass.txt

# 使用指定用户名和密码
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s ssh \
    -u admin \
    -p admin123

# 使用多个用户名（逗号分隔）
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s ssh \
    -u admin,root,test \
    -p admin123,password,123456

# 扫描网段并输出到 Excel
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s ssh \
    -c 20 \
    -o
```

### 2. Tomcat 弱口令扫描

```bash
# 扫描 Tomcat Manager（使用默认字典）
gxtools pentest weakpass -t 192.168.1.1 -s tomcat

# 使用特定用户名和密码
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s tomcat \
    -u tomcat \
    -p tomcat

# 扫描多个目标
gxtools pentest weakpass \
    -t 192.168.1.1,192.168.1.2,192.168.1.3 \
    -s tomcat \
    -u tomcat,admin \
    -p tomcat,admin,s3cret \
    -c 5
```

### 3. Nacos 弱口令扫描

```bash
# 扫描 Nacos 服务（默认字典）
gxtools pentest weakpass -t 192.168.1.1 -s nacos

# 使用 Nacos 默认凭证
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s nacos \
    -u nacos \
    -p nacos

# 批量扫描
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s nacos \
    -u nacos,admin \
    -p nacos,admin123 \
    -c 10 \
    -o
```

### 4. 全服务扫描

```bash
# 扫描所有支持的服务
gxtools pentest weakpass -t 192.168.1.1 -s all

# 扫描网段（所有服务）
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s all \
    -c 15 \
    -T 10 \
    -o
```

---

## 📝 参数格式说明

### 方式 1: 使用字典文件（.txt）

**用户名字典 (usernames.txt)**:
```
# 注释行以 # 开头
# 一行一个用户名

admin
root
administrator
test
user
tomcat
nacos
```

**密码字典 (passwords.txt)**:
```
# 注释行以 # 开头
# 一行一个密码
# 空行表示空密码

admin
123456
password
admin123
root
tomcat
nacos
```

**使用示例**:
```bash
gxtools pentest weakpass -t 192.168.1.1 -s ssh -u users.txt -p pass.txt
```

### 方式 2: 使用字符串

**单个值**:
```bash
# 单个用户名和密码
gxtools pentest weakpass -t 192.168.1.1 -s ssh -u admin -p admin123
```

**多个值（逗号分隔）**:
```bash
# 多个用户名和密码
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s ssh \
    -u admin,root,test \
    -p admin123,password,123456
```

### 方式 3: 混合使用

```bash
# 用户名使用字符串，密码使用字典文件
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s ssh \
    -u admin,root \
    -p passwords.txt

# 用户名使用字典文件，密码使用字符串
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s tomcat \
    -u usernames.txt \
    -p tomcat,admin123
```

### 默认字典

如果未提供字典文件，工具会使用内置的默认字典：

**默认用户名**:
- admin, root, administrator, user, test
- nacos, tomcat, weblogic, jboss
- mysql, redis, postgres, nginx

**默认密码**:
- admin, password, 123456, admin123
- root, nacos, tomcat, weblogic
- 空密码 (尝试无密码登录)

---

## 📊 输出示例

### 终端输出

```
🔍 开始弱口令扫描...
📚 加载字典: 7 个用户名, 9 个密码
🎯 目标服务: ssh
⚙️  配置: 并发=10, 超时=5秒
🔍 扫描任务: 1 个目标 × 7 个用户 × 9 个密码 = 63 个任务
[00:00:15] [████████████████████] 63/63 (100%) [ETA: 0s]
  ✅ 发现弱口令: 192.168.1.1:22 [SSH] admin:admin123
✅ 弱口令扫描完成

📊 扫描统计:
   总任务: 63
   发现弱口令: 1 个
   耗时: 15.32s

🔓 弱口令详情:
   192.168.1.1:22 [SSH] admin:admin123 - SSH认证成功
```

### Excel 输出

导出的 Excel 文件包含以下列：

| IP地址 | 端口 | 服务 | 用户名 | 密码 | 状态 | 详情 |
|--------|------|------|--------|------|------|------|
| 192.168.1.1 | 22 | SSH | admin | admin123 | 成功 | SSH认证成功 |
| 192.168.1.2 | 8080 | Tomcat | tomcat | tomcat | 成功 | Tomcat Manager 访问成功 |

文件保存路径: `output/weakpass/weakpass_YYYYMMDD_HHMMSS.xlsx`

---

## ⚙️ 性能优化

### 并发控制

```bash
# 低并发（适合网络较慢的环境）
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh -c 5

# 中等并发（推荐）
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh -c 10

# 高并发（适合内网高速环境）
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh -c 20
```

⚠️ **注意**: 并发数过高可能导致：
- 目标系统负载过高
- 触发防火墙/IDS 告警
- 账户被锁定（某些系统有登录失败限制）

### 超时设置

```bash
# 短超时（快速扫描）
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh -T 3

# 中等超时（推荐）
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh -T 5

# 长超时（网络不稳定时）
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh -T 10
```

---

## 🛡️ 安全注意事项

### 1. 授权测试
- ✅ 仅在获得授权的系统上使用
- ✅ 保留授权文件和测试记录
- ❌ 禁止未经授权的扫描

### 2. 账户锁定
- 某些系统在多次登录失败后会锁定账户
- 建议：
  - 降低并发数
  - 减少密码字典大小
  - 增加扫描间隔

### 3. 日志记录
- 目标系统会记录所有登录尝试
- 建议与系统管理员沟通
- 在测试窗口期内进行

### 4. 网络影响
- 大量扫描可能影响网络性能
- 建议在非业务高峰期进行
- 监控网络流量

---

## 🔧 高级用法

### 1. 分段扫描

```bash
# 分批扫描大型网段
gxtools pentest weakpass -t 192.168.1.1-50 -s ssh -o
gxtools pentest weakpass -t 192.168.1.51-100 -s ssh -o
gxtools pentest weakpass -t 192.168.1.101-150 -s ssh -o
```

### 2. 服务组合扫描

```bash
# 先扫描 SSH
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh -o

# 再扫描 Web 服务
gxtools pentest weakpass -t 192.168.1.0/24 -s web -o

# 最后扫描中间件
gxtools pentest weakpass -t 192.168.1.0/24 -s tomcat -o
gxtools pentest weakpass -t 192.168.1.0/24 -s nacos -o
```

### 3. 自定义扫描策略

**策略 A: 使用字典文件（适合大量凭证）**
```bash
# 创建自定义字典
echo "admin" > company_users.txt
echo "it_admin" >> company_users.txt
echo "developer" >> company_users.txt

echo "CompanyName123" > company_pass.txt
echo "CompanyName@2024" >> company_pass.txt
echo "Welcome123" >> company_pass.txt

# 执行扫描
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s all \
    -u company_users.txt \
    -p company_pass.txt \
    -o
```

**策略 B: 使用字符串（适合快速测试）**
```bash
# 快速测试常见凭证
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s ssh \
    -u admin,root \
    -p CompanyName123,CompanyName@2024,Welcome123 \
    -o
```

**策略 C: 针对特定应用**
```bash
# 针对 Tomcat
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s tomcat \
    -u tomcat,admin,manager \
    -p tomcat,s3cret,admin123 \
    -o

# 针对 Nacos
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s nacos \
    -u nacos \
    -p nacos,nacos123 \
    -o
```

---

## 🐛 故障排查

### 问题 1: 字典文件未找到

**症状**: `⚠️ 字典文件不存在: usernames.txt，使用默认字典`

**解决方案 A - 创建字典文件**:
```bash
# 创建字典文件
touch usernames.txt passwords.txt
echo "admin" > usernames.txt
echo "admin123" > passwords.txt
```

**解决方案 B - 使用绝对路径**:
```bash
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s ssh \
    -u /path/to/usernames.txt \
    -p /path/to/passwords.txt
```

**解决方案 C - 直接使用字符串（推荐）**:
```bash
# 不需要创建文件，直接指定
gxtools pentest weakpass \
    -t 192.168.1.1 \
    -s ssh \
    -u admin,root \
    -p admin123,password
```

### 问题 2: 连接超时

**症状**: 扫描很慢或一直无结果

**解决**:
```bash
# 增加超时时间
gxtools pentest weakpass -t 192.168.1.1 -s ssh -T 10

# 降低并发数
gxtools pentest weakpass -t 192.168.1.1 -s ssh -c 5
```

### 问题 3: SSH 认证失败

**症状**: 所有 SSH 尝试都失败

**可能原因**:
- 目标未开放 SSH 服务
- 防火墙阻止连接
- SSH 配置了密钥认证（不支持密码）
- 账户已被锁定

**排查**:
```bash
# 先测试端口是否开放
gxtools pentest scan -t 192.168.1.1 -p 22

# 手动测试 SSH 连接
ssh admin@192.168.1.1
```

### 问题 4: Tomcat/Nacos 无法检测

**症状**: 明确有弱口令但未检测到

**可能原因**:
- URL 路径不标准
- 需要 HTTPS 而非 HTTP
- 需要特定的请求头

**排查**:
```bash
# 手动测试
curl -u admin:admin http://192.168.1.1:8080/manager/html
curl http://192.168.1.1:8848/nacos/
```

---

## 📈 性能参考

### 扫描速度估算

| 场景 | 目标数 | 用户数 | 密码数 | 并发 | 预计时间 |
|------|--------|--------|--------|------|----------|
| 单机 SSH | 1 | 10 | 10 | 10 | ~10秒 |
| 小型网段 SSH | 10 | 10 | 10 | 10 | ~1分钟 |
| C段 SSH | 254 | 10 | 10 | 20 | ~10分钟 |
| C段全服务 | 254 | 10 | 10 | 15 | ~30分钟 |

*实际时间取决于网络延迟、目标响应速度等因素*

### 资源占用

- **CPU**: 低（主要是网络 I/O）
- **内存**: 中等（~100-500MB）
- **网络**: 中等（每秒数十到数百个连接）

---

## 📚 相关文档

- [OPTIMIZATION.md](OPTIMIZATION.md) - 代码优化详情
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - 快速参考指南
- [BUILD_GUIDE.md](BUILD_GUIDE.md) - 编译部署指南
- [TODO.md](TODO.md) - 后续功能规划

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request：

### 希望支持更多服务？

请提供：
- 服务名称和默认端口
- 认证方式（HTTP Basic / Form / API）
- 认证接口 URL
- 成功/失败判断标识

### 改进建议

- 性能优化建议
- 检测准确率改进
- 新功能需求

---

## ⚠️ 免责声明

本工具仅用于合法的安全测试目的。使用本工具进行任何未经授权的测试都是违法的。

- 使用者需对自己的行为负责
- 开发者不承担任何法律责任
- 请遵守当地法律法规
- 仅在授权范围内使用

---

**版本**: 1.0  
**最后更新**: 2024年  
**维护者**: GX Tools Team