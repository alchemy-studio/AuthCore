```bash
$ cargo build
```

编译结果：

```bash
weli@192:~/w/huiwing|main⚡*?
➤ pwd                                                                                                                                                                                                                                                                                                                                                              23:45:35
/Users/weli/works/huiwing
weli@192:~/w/huiwing|main⚡*?
➤ ls target/debug/upyun_tool                                                                                                                                                                                                                                                                                                                                       23:45:39
target/debug/upyun_tool
```

拷贝工具：

```bash
➤ sudo cp target/debug/upyun_tool /usr/local/bin/                                                                                                                                                                                                                                                                                                                  23:46:05
```

在`upyun_tool`的目录下运行下面的命令测试：

```bash
$ pwd
/Users/weli/works/huiwing/upyun_tool
```


测试上传：

```bash
./upload_assets.sh # 默认上传`local-dev`
./upload_assets.sh ./.alchemy_upyun_pass # 尽量不尝试正式服务器
./upload_assets.sh ./.moicen_upyun_pass
```

---

测试下载：

```bash
weli@192:~/w/h/upyun_tool|main⚡?
➤ upyun_tool --upt-download https://upyun.huiwings.cn/music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4 --config ./.huiwings_upt_config                                                                                                                                                       22:54:45
Command value for upload: false
Command value for download: false
Command value for delete: false
Command value for upt_download: true
Command value for remote file: https://upyun.huiwings.cn/music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4
Command value for config: ./.huiwings_upt_config
 ---=== Debug ===--- 
Debug : upload = false
Debug : download = false
Debug : delete = false
Debug : upt_download = true
Debug : remote_file_string = https://upyun.huiwings.cn/music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4
Debug : local_file_string = 
Debug : config_file = ./.huiwings_upt_config
Debug : server_index = 0
User : C5E4B01EC86A4CE8A84871EA2C826DD1 length : 32
Passwd : 3600 length : 4
Bucket :  length : 0
Upt url : https://upyun.huiwings.cn/music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4
Upt uri : /music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4
Secret : C5E4B01EC86A4CE8A84871EA2C826DD1 length : 32
Duration : 3600 length : 4
Duration : 3600
Debug - secret : C5E4B01EC86A4CE8A84871EA2C826DD1
Debug - duration : 3600
Debug - uri : /music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4
End time : 1726070095
MD5 string : C5E4B01EC86A4CE8A84871EA2C826DD1&1726070095&/music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4
Signature : f7e4dc1a1e7a117e5f836821146814c9
Upt string : 117e5f831726070095
Upt request url : https://upyun.huiwings.cn/music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4?_upt=117e5f831726070095
Response {
    url: Url {
        scheme: "https",
        cannot_be_a_base: false,
        username: "",
        password: None,
        host: Some(
            Domain(
                "upyun.huiwings.cn",
            ),
        ),
        port: None,
        path: "/music-room/29a5a9eb-83a3-4a33-8cb0-2c3b74673983_compressed.mp4",
        query: Some(
            "_upt=117e5f831726070095",
        ),
        fragment: None,
    },
    status: 200,
    headers: {
        "server": "marco/3.2",
        "date": "Wed, 11 Sep 2024 14:54:55 GMT",
        "content-type": "video/mp4",
        "content-length": "40463974",
        "connection": "keep-alive",
        "x-request-id": "af83479ab88c409a940815c33ec33231",
        "x-source": "U/200",
        "x-upyun-content-length": "40463974",
        "etag": "\"d208cb44dac7146400088ce295fb2e21\"",
        "last-modified": "Thu, 05 Sep 2024 16:01:06 GMT",
        "x-upyun-content-type": "video/mp4",
        "x-slice-complete-length": "40463974",
        "x-slice-etag": "d208cb44dac7146400088ce295fb2e21",
        "x-slice-size": "1048576",
        "expires": "Thu, 19 Sep 2024 14:54:55 GMT",
        "cache-control": "max-age=691200",
        "accept-ranges": "bytes",
        "age": "0",
        "via": "T.208.M, V.403-zj-fud-208, S.mix-hz-fdi1-214, T.214.M, V.mix-hz-fdi1-217, T.194.M, M.cun-he-sjw8-194",
    },
}
Upt download success .
```