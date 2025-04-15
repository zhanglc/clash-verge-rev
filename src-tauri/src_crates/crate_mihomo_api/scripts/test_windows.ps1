$pipeName = "\\.\pipe\mihomo"
$pipe = new-object System.IO.Pipes.NamedPipeClientStream(".", "mihomo", [System.IO.Pipes.PipeDirection]::InOut)
$pipe.Connect(1000)  # 尝试连接 1 秒
if ($pipe.IsConnected) {
    Write-Host "成功连接到管道"
    # 示例写入或读取可以加上如下内容
    # $writer = new-object System.IO.StreamWriter($pipe)
    # $writer.WriteLine("hello pipe")
    # $writer.Flush()
    $pipe.Close()
} else {
    Write-Host "连接失败"
}
