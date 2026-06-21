$client = New-Object System.Net.Sockets.TcpClient('127.0.0.1',5000)
$stream = $client.GetStream()
$sr = New-Object System.IO.StreamReader $stream
$sw = New-Object System.IO.StreamWriter $stream
$sw.AutoFlush = $true
$token = $sr.ReadLine()
Write-Host "Token received: $token"
$sw.WriteLine($token)
$auth = $sr.ReadLine()
Write-Host "Auth reply: $auth"
$sw.WriteLine('PING')
$pong = $sr.ReadLine()
Write-Host "PING reply: $pong"
$sw.WriteLine('GET_SCENE')
$scene = $sr.ReadLine()
Write-Host "GET_SCENE reply: $scene"
$client.Close()
