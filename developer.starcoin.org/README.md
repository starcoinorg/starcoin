# Starcoin 开发者网站

### PreRequirement

1. Install [hugo](https://gohugo.io/getting-started/installing/) > 0.68.0

### Checkout

```shell script
git clone https://github.com/starcoinorg/website
```


### 运行server
- 开发运行
```
hugo server --minify
```
- 通过 http://localhost:1313/ 查看效果， 生成的文件放置于 public 目录下。



### 编译发布
- 项目根目录执行`hugo`即可编译静态文件，编译后在根目录生成`public`文件夹，将public部署至指定地点即可直接访问。
- 注：编译前修改`config.toml`中baseURL参数对应当前环境域名。
test
