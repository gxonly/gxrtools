# 弱口令扫描快速示例

> 本文档提供弱口令扫描模块的实用示例，帮助您快速上手。

---

## 🚀 快速开始

### 最简单的使用方式

```bash
# 扫描单个IP的SSH服务（使用默认字典）
gxtools pentest weakpass -t 192.168.1.1 -s ssh

# 使用特定用户名和密码
gxtools pentest weakpass -t 192.168.1.1 -s ssh -u admin -p admin123
```

---

## 📚 常见场景示例

### 场景 1: 快速测试单个主机

**需求**: 测试一台服务器的SSH弱口令

```bash
# 方式1: 使用默认字典（44个用户名 × 111个密码）
gxtools pentest weakpass -t 192.168.1.100 -s ssh

# 方式2: 只测试admin用户的常见密码
gxtools pentest weakpass \
    -t 192.168.1.100 \
    -s ssh \
    -u admin \
    -p admin,admin123,password,123456

# 方式3: 测试几个常见用户
gxtools pentest weakpass \
    -t 192.168.1.100 \
    -s ssh \
    -u admin,root,test \
    -p admin123,password
```

### 场景 2: 扫描整个网段

**需求**: 扫描C段的SSH服务

```bash
# 使用默认字典扫描
gxtools pentest weakpass -t 192.168.1.0/24 -s ssh -c 20 -o

# 只测试最常见的凭证（快速模式）
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s ssh \
    -u admin,root \
    -p admin,admin123,123456 \
    -c 20 \
    -o
```

### 场景 3: Tomcat Manager 检测

**需求**: 检测Tomcat管理界面弱口令

```bash
# 使用Tomcat默认凭证
gxtools pentest weakpass \
    -t 192.168.1.50 \
    -s tomcat \
    -u tomcat \
    -p tomcat

# 测试多个常见凭证
gxtools pentest weakpass \
    -t 192.168.1.50 \
    -s tomcat \
    -u tomcat,admin,manager \
    -p tomcat,s3cret,admin123,manager

# 批量扫描多台服务器
gxtools pentest weakpass \
    -t 192.168.1.50,192.168.1.51,192.168.1.52 \
    -s tomcat \
    -u tomcat,admin \
    -p tomcat,admin123 \
    -o
```

### 场景 4: Nacos 弱口令检测

**需求**: 检测Nacos服务的弱口令

```bash
# Nacos默认凭证（nacos/nacos）
gxtools pentest weakpass \
    -t 192.168.1.80 \
    -s nacos \
    -u nacos \
    -p nacos

# 测试多个可能的密码
gxtools pentest weakpass \
    -t 192.168.1.80 \
    -s nacos \
    -u nacos,admin \
    -p nacos,admin,admin123,nacos123

# 扫描整个网段
gxtools pentest weakpass \
    -t 192.168.1.0/24 \
    -s nacos \
    -u nacos \
    -p nacos,nacos123 \
    -c 10 \
    -o
```

### 场景 5: 扫描多种服务

**需求**: 一次性扫描所有支持的服务

```bash
# 扫描所有服务（使用默认字典）
gxtools pentest weakpass -t 192.168.1.100 -s all -o

# 使用精简的凭证列表（快速模式）
gxtools pentest weakpass \
    -t 192.168.1.100 \
    -s all \
    -u admin,root,tomcat,nacos \
    -p admin123,password,tomcat,nacos \
    -c 15 \
    -o
```

### 场景 6: 企业内网扫描

**需求**: 扫描企业内网多个网段

```bash
# 方式1: 逐个网段扫描
gxtools pentest weakpass -t 10.0.1.0/24 -s ssh -u admin,root -p CompanyPass2024 -o
gxtools pentest weakpass -t 10.0.2.0/24 -s ssh -u admin,root -p CompanyPass2024 -o
gxtools pentest weakpass -t 10.0.3.0/24 -s ssh -u admin,root -p CompanyPass2024 -o

# 方式2: 使用自定义字典文件
echo "admin" > company_users.txt
echo "it_admin" >> company_users.txt
echo "devops" >> company_users.txt

echo "CompanyPass2024" > company_pass.txt
echo "Company@2024" >> company_pass.txt
echo "Welcome123" >> company_pass.txt

gxtools pentest weakpass \
    -t 10.0.0.0/16 \
    -s all \
    -u company_users.txt \
    -p company_pass.txt \
    -c 20 \
    -o
```

---

## 🎯 参数组合推荐

### 快速扫描（少量凭证）

```bash
# 适合：快速验证、应急响应
gxtools pentest weakpass \
    -t TARGET \
    -s SERVICE \
    -u admin,root \
    -p admin123,password \
    -c 20 \
    -T 3
```

**特点**:
- 凭证少，速度快
- 只测试最常见的凭证
- 适合快速筛查

### 标准扫描（使用字典文件）

```bash
# 适合：常规安全测试
gxtools pentest weakpass \
    -t TARGET \
    -s SERVICE \
    -u usernames.txt \
    -p passwords.txt \
    -c 10 \
    -T 5 \
    -o
```

**特点**:
- 使用完整字典
- 覆盖面广
- 适合正式测试

### 深度扫描（大字典）

```bash
# 适合：深度安全审计
gxtools pentest weakpass \
    -t TARGET \
    -s SERVICE \
    -u big_users.txt \
    -p big_pass.txt \
    -c 5 \
    -T 10 \
    -o
```

**特点**:
- 大型字典
- 低并发，避免触发防护
- 时间较长

---

## 💡 实用技巧

### 技巧 1: 分段扫描大型网络

```bash
#!/bin/bash
# 分段扫描脚本

for i in {1..10}; do
    echo "扫描第 $i 段..."
    gxtools pentest weakpass \
        -t 192.168.$i.0/24 \
        -s ssh \
        -u admin,root \
        -p admin123,password \
        -c 20 \
        -o
    sleep 10
done
```

### 技巧 2: 针对性扫描

```bash
# 先端口扫描找出开放的服务
gxtools pentest scan -t 192.168.1.0/24 -p 22,8080,8848 -o

# 根据结果针对性扫描
# 发现SSH开放的主机
gxtools pentest weakpass -t 192.168.1.5,192.168.1.8 -s ssh -u admin -p admin123

# 发现Tomcat的主机
gxtools pentest weakpass -t 192.168.1.10,192.168.1.20 -s tomcat -u tomcat -p tomcat
```

### 技巧 3: 使用输出结果

```bash
# 导出到Excel
gxtools pentest weakpass -t 192.168.1.0/24 -s all -o

# Excel文件保存在: output/weakpass/weakpass_YYYYMMDD_HHMMSS.xlsx
# 可以用Excel打开查看详细结果
```

### 技巧 4: 组合多个命令

```bash
# 完整的安全测试流程
# 1. 主机存活探测
gxtools net ping -t 192.168.1.0/24 -o

# 2. 端口扫描
gxtools pentest scan -t 192.168.1.0/24 --live -o

# 3. 弱口令扫描
gxtools pentest weakpass -t 192.168.1.0/24 -s all -o
```

---

## 🎨 字典文件示例

### 最小化字典（快速测试）

**mini_users.txt**:
```
admin
root
test
```

**mini_pass.txt**:
```
admin
admin123
123456
password
```

使用：
```bash
gxtools pentest weakpass -t TARGET -s ssh -u mini_users.txt -p mini_pass.txt
```

### 中等字典（标准测试）

**users.txt**:
```
admin
administrator
root
user
test
tomcat
nacos
```

**pass.txt**:
```
admin
admin123
password
123456
root
test
tomcat
nacos
```

### 应用专用字典

**tomcat_users.txt**:
```
tomcat
admin
manager
role1
both
```

**tomcat_pass.txt**:
```
tomcat
s3cret
manager
admin
password
```

**nacos_creds.txt**:
```
nacos:nacos
admin:admin
admin:nacos
nacos:admin123
```

---

## ⚡ 性能优化建议

### 建议 1: 合理设置并发

```bash
# 内网高速环境
-c 20

# 外网或不稳定网络
-c 5

# 防止触发防护系统
-c 3
```

### 建议 2: 调整超时时间

```bash
# 内网（响应快）
-T 3

# 标准（推荐）
-T 5

# 外网或响应慢
-T 10
```

### 建议 3: 精简字典

```bash
# 不推荐：使用超大字典
-u huge_users.txt -p huge_pass.txt  # 可能导致账户锁定

# 推荐：使用精选字典
-u admin,root,test -p admin123,password,123456  # 快速且有效
```

---

## 📊 输出格式示例

### 终端输出

```
🔍 开始弱口令扫描...
📚 加载字典: 3 个用户名, 4 个密码
🎯 目标服务: ssh
⚙️  配置: 并发=10, 超时=5秒
🔍 扫描任务: 1 个目标 × 3 个用户 × 4 个密码 = 12 个任务
[00:00:08] [████████████████████] 12/12 (100%) [ETA: 0s]
  ✅ 发现弱口令: 192.168.1.1:22 [SSH] admin:admin123
✅ 弱口令扫描完成

📊 扫描统计:
   总任务: 12
   发现弱口令: 1 个
   耗时: 8.45s

🔓 弱口令详情:
   192.168.1.1:22 [SSH] admin:admin123 - SSH认证成功
```

### Excel输出示例

| IP地址 | 端口 | 服务 | 用户名 | 密码 | 状态 | 详情 |
|--------|------|------|--------|------|------|------|
| 192.168.1.1 | 22 | SSH | admin | admin123 | 成功 | SSH认证成功 |
| 192.168.1.5 | 8080 | Tomcat | tomcat | tomcat | 成功 | Tomcat Manager 访问成功 |
| 192.168.1.8 | 8848 | Nacos | nacos | nacos | 成功 | Nacos 登录成功 |

---

## 🔒 安全提示

### ⚠️ 使用前必读

1. **获得授权**: 仅在授权范围内使用
2. **避免误伤**: 注意账户锁定策略
3. **控制并发**: 避免对目标系统造成压力
4. **保护结果**: 扫描结果包含敏感信息，妥善保管

### 建议的安全实践

```bash
# 1. 先小范围测试
gxtools pentest weakpass -t 192.168.1.1 -s ssh -u admin -p admin123

# 2. 确认无问题后扩大范围
gxtools pentest weakpass -t 192.168.1.1-10 -s ssh -u admin -p admin123

# 3. 最后进行完整扫描
gxtools pentest weakpass -t 192.168.1.0/24 -s all -o

# 4. 扫描结束后，及时通知管理员修改弱口令
```

---

## 📞 常见问题

**Q: 为什么扫描很慢？**
- A: 降低并发数 `-c 5`，或增加超时时间 `-T 10`

**Q: 如何只测试一个用户名？**
- A: 直接指定字符串 `-u admin -p admin123,password,123456`

**Q: 支持空密码吗？**
- A: 支持，在字典文件中添加空行，或在字符串中留空（但不推荐）

**Q: 如何测试多个服务？**
- A: 使用 `-s all` 或分别执行多次扫描

**Q: 结果保存在哪里？**
- A: `output/weakpass/weakpass_YYYYMMDD_HHMMSS.xlsx`

---

## 🔗 相关文档

- [完整使用指南](WEAKPASS_GUIDE.md) - 详细功能说明
- [快速参考](QUICK_REFERENCE.md) - 所有模块快速参考
- [编译部署](BUILD_GUIDE.md) - 安装和部署指南

---

**最后更新**: 2024年  
**维护者**: GX Tools Team