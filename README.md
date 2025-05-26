# gxrtools

## Ping

默认结果存储至output/ping/日期.xlsx中

~~~bash
# 参数
执行 ping 操作

Usage: gxtools.exe ping [OPTIONS] --target <TARGET>

Options:
  -t, --target <TARGET>            IP地址或网段（CIDR），如：192.168.1.1 或 192.168.1.0/24
  -T, --timeout <TIMEOUT>          超时时间（秒） [default: 2]
  -c, --concurrency <CONCURRENCY>  最大并发数 [default: 100]
  -e, --echo                       是否打印结果到终端
  -h, --help                       Print help

# 例子
gxtools.exe ping -t 192.168.100.1,192.168.100.3-5,192.168.200.1/24
~~~

## Linux（ssh方式）

默认存储于output/ssh/ip.json

~~~bash
# 参数
Usage: gxtools.exe linux [OPTIONS]

Options:
  -H, --host <HOST>                        远程主机的IP地址 (与 -f 互斥)
  -f, --file <FILE>                        从Excel文件读取主机列表(格式: 主机,端口,用户名,密码/密钥路径) (与 -H 互斥)
  -P, --port <PORT>                        SSH端口号 (当使用 -H 时有效) [default: 22]
  -u, --username <USERNAME>                用户名 (当使用 -H 时有效) [default: root]
  -p, --password-or-key <PASSWORD_OR_KEY>  密码或私钥路径 (当使用 -H 时必需)
  -c, --commands <COMMANDS>...             要执行的命令
  -t, --threads <THREADS>                  并发线程数 [default: 4]
  -e, --echo                               输出到控制台，使用前提需指定自定义命令
  -h, --help                               Print help
  
# 例子
gxtools.exe linux -H 192.168.100.1 -P 22 -p mima -u root -e -c "pwd"
gxtools.exe linux -f linux.xlsx		# 默认命令
gxtools.exe linux -f linux.xlsx -c "ls" -e

~~~

## windows

默认存储于output/windows/ip.json

~~~bash
# 参数
Usage: gxtools.exe windows [OPTIONS]

Options:
  -f, --file <FILE>  指定ps1脚本路径
  -p, --port <PORT>  修改端口，默认3000 [default: 3000]
  -i, --ip <IP>      绑定本机IP，默认自动识别，多网卡可能异常
  -h, --help         Print help

# 例子
gxtools.exe windows		# 默认运行，自动识别网卡，使用本机3000端口
gxtools.exe windows -i 192.168.1.1 -p 12321		# 绑定网卡，并使用12321端口
~~~

## MySQL（待定）