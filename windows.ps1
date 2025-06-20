# 获取密码策略及审核策略
# 1. 导出安全配置到临时文件
$cfgFilePath = "$env:TEMP\secpol.cfg"
secedit /export /cfg $cfgFilePath /quiet

# 2. 读取临时配置文件
$secpolContent = Get-Content $cfgFilePath

# 3. 定义一个变量来存储当前读取的节的名称
$currentSection = ""

# 4. 创建一个哈希表来存储相关部分的数据
$systemAccess = @{}
$eventAudit = @{}

# 5. 循环读取文件内容并筛选出特定部分
foreach ($line in $secpolContent) {
    # 如果是新的节，更新当前节名称
    if ($line -match "^\[(.*)\]") {
        $currentSection = $matches[1]
    }

    # 如果当前节是 [System Access]，则保存这一部分的内容
    if ($currentSection -eq "System Access") {
        if ($line.Trim() -ne "" -and $line -notmatch "^\[.*\]") {
            $keyValue = $line -split "="
            if ($keyValue.Length -eq 2) {
                $systemAccess[$keyValue[0].Trim()] = $keyValue[1].Trim()
            }
        }
    }

    # 如果当前节是 [Event Audit]，则保存这一部分的内容
    if ($currentSection -eq "Event Audit") {
        if ($line.Trim() -ne "" -and $line -notmatch "^\[.*\]") {
            $keyValue = $line -split "="
            if ($keyValue.Length -eq 2) {
                $eventAudit[$keyValue[0].Trim()] = $keyValue[1].Trim()
            }
        }
    }
}

# 获取进程
$processInfo = Get-Process | Select-Object Name
# 获取服务
$serviceInfo = Get-Service | Select-Object Status, DisplayName
# 获取端口
$portInfo = Get-NetTCPConnection -State Listen | Select-Object LocalPort,LocalAddress

# 获取屏幕保护程序配置（当前用户）
$screenSaverSettings = @{
    # 是否启用
    ScreenSaverEnabled   = (Get-ItemProperty -Path "HKCU:\Control Panel\Desktop" -Name ScreenSaveActive -ErrorAction SilentlyContinue).ScreenSaveActive
    # 是否需要密码
    PasswordOnResume     = (Get-ItemProperty -Path "HKCU:\Control Panel\Desktop" -Name ScreenSaverIsSecure -ErrorAction SilentlyContinue).ScreenSaverIsSecure
    # 超时推出时间
    IdleTimeInSeconds    = (Get-ItemProperty -Path "HKCU:\Control Panel\Desktop" -Name ScreenSaveTimeOut -ErrorAction SilentlyContinue).ScreenSaveTimeOut
}

# 获取所有本地组
$groups = Get-WmiObject -Class Win32_Group -Filter "LocalAccount='TRUE'"

# 获取共享目录
$shareDir = Get-SmbShare | Select-Object Name, Path, Description, ScopeName, FolderEnumerationMode, EncryptData | ConvertTo-Json -Depth 3 -Compress

# 主机信息
$os = Get-CimInstance Win32_OperatingSystem
$cs = Get-CimInstance Win32_ComputerSystem
$bios = Get-CimInstance Win32_BIOS

$systemInfo = @{
    ComputerName = $env:COMPUTERNAME
    OS = $os.Caption
    OSVersion = $os.Version
    Architecture = $os.OSArchitecture
    Manufacturer = $cs.Manufacturer
    Model = $cs.Model
    BIOSVersion = $bios.SMBIOSBIOSVersion
    BIOSReleaseDate = $bios.ReleaseDate
    SystemType = $cs.SystemType
    TotalPhysicalMemory = $cs.TotalPhysicalMemory
}

# 安装软件情况
$sofeWare = Get-ItemProperty @(
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKLM:\Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
) | Where-Object { $_.DisplayName } |
  Select-Object DisplayName, DisplayVersion, Publisher, InstallDate |
  Sort-Object DisplayName

# 初始化结果数组
$groupMembers = @()

foreach ($group in $groups) {
    # 获取该组的成员（通过 GroupComponent）
    $members = Get-WmiObject -Class Win32_GroupUser | Where-Object {
        $_.GroupComponent -like "*Name=`"$($group.Name)`"*" 
    }

    if ($members.Count -eq 0) {
        $memberNames = @()
    } else {
        # 提取成员名称（从 PartComponent 提取用户/组名）
        $memberNames = $members | ForEach-Object {
            if ($_ -and $_.PartComponent -match 'Name="([^"]+)"') {
                $matches[1]
            }
        }
    }

    $groupMembers += [PSCustomObject]@{
        GroupName = $group.Name
        Members = $memberNames
    }
}


$combinedResult = [PSCustomObject]@{
    SystemAccess = $systemAccess
    EventAudit = $eventAudit
    Processes = $processInfo
    Services  = $serviceInfo
    ScreenSaverConfig = $screenSaverSettings
    GroupMembers = $groupMembers
    PortListen = $portInfo
    ShareDir = $shareDir
    SystemInfo = $systemInfo
    SofeWare = $sofeWare
}

$commandResult = $combinedResult | ConvertTo-Json -Depth 3 -Compress


# 接口地址
$url = "http://10.10.10.110:3000/report"

# 构造请求头（如需授权，可以加 Authorization）
$headers = @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer YOUR_API_TOKEN"  # 如不需要可去掉此行
}

# 发送 POST 请求
$response = Invoke-RestMethod -Uri $url -Method POST -Headers $headers -Body $commandResult

# 打印响应
Write-Output "Server response:"
Write-Output $response
