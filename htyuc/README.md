## 项目测试
* 导入环境变量
```shell script
export $(grep -v '^#' .test | xargs)
```
test文件内容
```.env 
DATABASE_URL=postgres://username:password@localhost/db
OPERATOR_PWD=1
FORM_API_KEY=1
OPERATOR=1
WEIXIN_APP_ID=1
WEIXIN_SECRET=1
```

* 执行测试
```shell script
$  print_debug=true POOL_SIZE=1 cargo test -- --test-threads=1 --nocapture 
```

## USER DATABASE

* 启动数据库命令
```shell script
docker-compose -f container/docker-compose.yml up db
```

* 初始化数据库
```shell script
diesel setup
```

* 运行migrations
```shell script
diesel migration run
```

这个项目同时承担「微信接口」和「又拍云」的上传功能。
