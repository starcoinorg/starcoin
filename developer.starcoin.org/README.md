# Starcoin 开发者网站

### PreRequirement

1. Install [hugo](https://gohugo.io/getting-started/installing/) > 0.68.0

### Checkout

```shell script
git clone https://github.com/starcoinorg/starcoin
git submodule update --init
```


### Run server
```
hugo -s ./developer.starcoin.org server --minify
```
open http://localhost:1313/ in browser



### 编译发布
- 项目根目录执行`hugo`即可编译静态文件，编译后在根目录生成`public`文件夹，将public部署至指定地点即可直接访问。
- 注：编译前修改`config.toml`中baseURL参数对应当前环境域名。
